/**
 * Adapted from https://github.com/fourdollars/x11-touchscreen-calibrator/blob/master/x11-touchscreen-calibrator.c
 * Codyright (C) 2013 Shih-Yuan Lee (FourDollars) <sylee@canonical.com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a cody of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */
use std::{
    ffi::{CStr, CString},
    ptr,
};

use anyhow::{Ok, Result};

use x11::{xinput2::*, xlib::*, xrandr::XRRGetScreenSizeRange};
use xrandr::{Output, Rotation, ScreenResources, XHandle};

use super::x_display::XDisplay;

// The main handle consists simply of a pointer to the display
type DisplayHandleSys = ptr::NonNull<Display>;
#[derive(Debug)]
pub struct XDisplayHandle {
    sys: DisplayHandleSys,
}

impl XDisplayHandle {
    pub fn open() -> Self {
        Self {
            sys: ptr::NonNull::new(unsafe { XOpenDisplay(ptr::null()) }).unwrap(),
        }
    }
}

impl Drop for XDisplayHandle {
    fn drop(&mut self) {
        unsafe { XCloseDisplay(self.sys.as_ptr()) };
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TouchCalibrationMode {
    PerDisplay,
    PreferEmbedded,
    PreferExternal,
}

#[derive(Debug, Clone)]
struct TouchInputIdentifier {
    name: String,
    deviceid: i32,
}

struct DisplayInfo {
    dx: i32,
    dy: i32,
    dw: u32,
    dh: u32,
    sw: i32,
    sh: i32,
    rot: Rotation,
}

pub fn reconfigure(
    touch_mode: TouchCalibrationMode,
    display: &XDisplayHandle,
    xrandr: &mut XHandle,
    xdisplay: &mut XDisplay,
) -> Result<()> {
    let deck = xdisplay
        .get_embedded_output()?
        .filter(|v| v.connected && v.current_mode.is_some());
    let external = xdisplay
        .get_preferred_external_output()?
        .filter(|v| v.connected && v.current_mode.is_some());

    if deck.is_none() && external.is_none() {
        println!("no displays found, not configuring touch...");
        return Ok(());
    }

    let deck = deck.as_ref();
    let external = external.as_ref();

    let inputs = enumerate_touch_devices(display);

    let embedded_touch_name = "FTS3528:00 2808:1015";
    let deck_touch = inputs.iter().find(|v| v.name == embedded_touch_name);
    let external_touch = inputs.iter().find(|v| v.name != embedded_touch_name);

    // println!("inputs: {inputs:?}");

    reconfigure_internal(
        display,
        xrandr,
        deck,
        deck_touch,
        external,
        touch_mode,
        TouchCalibrationMode::PreferEmbedded,
    );
    reconfigure_internal(
        display,
        xrandr,
        external,
        external_touch,
        deck,
        touch_mode,
        TouchCalibrationMode::PreferExternal,
    );

    Ok(())
}

fn reconfigure_internal(
    display: &XDisplayHandle,
    xrandr: &mut XHandle,
    primary: Option<&Output>,
    primary_touch: Option<&TouchInputIdentifier>,
    secondary: Option<&Output>,
    touch_mode: TouchCalibrationMode,
    touch_mode_preference: TouchCalibrationMode,
) -> Result<()> {
    match (primary, primary_touch) {
        (Some(primary), Some(primary_touch)) => {
            if touch_mode == TouchCalibrationMode::PerDisplay || touch_mode == touch_mode_preference {
   TouchCalibrationTarget {
                        source_touch: primary_touch,
                        source_output: primary,
                        target_output: primary,
                    }
            } else {
TouchCalibrationTarget {
                    source_touch: primary_touch,
                    source_output: primary,
                    target_output: secondary.unwrap_or(primary),
                }
            }
            .reconfigure(display, xrandr);
        }
        (None, Some(primary_touch)) => {
            if let Some(secondary) = secondary {
                TouchCalibrationTarget {
                    source_touch: primary_touch,
                    source_output: secondary,
                    target_output: secondary,
                }
                .reconfigure(display, xrandr);
            } else {
                println!("no available display to map touch")
            }
        }
        (_, None) => println!("touch not detected, not configuring"),
    }

    Ok(())
}

impl DisplayInfo {
    fn new(
        display: &XDisplayHandle,
        handle: &mut XHandle,
        source_output: &Output,
        target_output: &Output,
    ) -> Self {
        let res = ScreenResources::new(handle).unwrap();

        let (sw, sh) = get_screen_info(&display);

        let source_crtc = res.crtc(handle, source_output.crtc.unwrap()).unwrap();
        let source_rot = source_crtc.rotation;

        let target_crtc = res.crtc(handle, target_output.crtc.unwrap()).unwrap();
        let target_mode = target_output
            .current_mode
            .map(|id| res.mode(id))
            .unwrap()
            .unwrap();

        let target_rot = target_crtc.rotation;
        let mut dw = target_mode.width;
        let mut dh = target_mode.height;

        if matches!(target_rot, Rotation::Left | Rotation::Right) {
            dw ^= dh;
            dh ^= dw;
            dw ^= dh;
        }

        Self {
            sh,
            sw,

            dw,
            dh,
            dx: target_crtc.x,
            dy: target_crtc.y,
            rot: source_rot,
        }
    }
}

struct TouchCalibrationTarget<'a> {
    source_touch: &'a TouchInputIdentifier,
    source_output: &'a Output,
    target_output: &'a Output,
}

impl<'a> TouchCalibrationTarget<'a> {
    fn reconfigure(&self, display: &XDisplayHandle, xrandr: &mut XHandle) {
        let mut display_info =
            DisplayInfo::new(display, xrandr, &self.source_output, &self.target_output);
        let deviceid = self.source_touch.deviceid;

        // TODO::there is a chance that the scaling mode of the display itself may affect the output.
        // Hoping that isn't the case

        scaling_full_mode(&display, deviceid, &mut display_info)
    }
}

fn scaling_full_mode(display: &XDisplayHandle, deviceid: i32, display_info: &mut DisplayInfo) {
    let d = display_info;

    let shift = [
        1.,
        0.,
        (*d).dx as f32 / (*d).sw as f32,
        0.,
        1.,
        (*d).dy as f32 / (*d).sh as f32,
        0.,
        0.,
        1.,
    ];
    let zoom = [
        (*d).dw as f32 / (*d).sw as f32,
        0.,
        0.,
        0.,
        (*d).dh as f32 / (*d).sh as f32,
        0.,
        0.,
        0.,
        1.,
    ];
    let m = multiply(&shift, &zoom);
    let m = rotate_reflect(&m, d);

    apply_matrix(display, deviceid, &m);
}

fn rotate_reflect(m: &[f32; 9], display_info: &mut DisplayInfo) -> [f32; 9] {
    let rotation = display_info.rot;

    let t = match rotation {
        Rotation::Normal => [1., 0., 0., 0., 1., 0., 0., 0., 1.], //0°
        Rotation::Inverted => [-1., 0., 1., 0., -1., 1., 0., 0., 1.], // 180°
        Rotation::Left | Rotation::Right => {
            if rotation == Rotation::Left {
                [0., -1., 1., 1., 0., 0., 0., 0., 1.] // 90°
            } else {
                [0., 1., 0., -1., 0., 1., 0., 0., 1.] // 270°
            }
        }
    };

    multiply(m, &t)
}

fn multiply(a: &[f32; 9], b: &[f32; 9]) -> [f32; 9] {
    let mut m = [0.; 9];

    m[0] = a[0] * b[0] + a[1] * b[3] + a[2] * b[6];
    m[1] = a[0] * b[1] + a[1] * b[4] + a[2] * b[7];
    m[2] = a[0] * b[2] + a[1] * b[5] + a[2] * b[8];
    m[3] = a[3] * b[0] + a[4] * b[3] + a[5] * b[6];
    m[4] = a[3] * b[1] + a[4] * b[4] + a[5] * b[7];
    m[5] = a[3] * b[2] + a[4] * b[5] + a[5] * b[8];
    m[6] = a[6] * b[0] + a[7] * b[3] + a[8] * b[6];
    m[7] = a[6] * b[1] + a[7] * b[4] + a[8] * b[7];
    m[8] = a[6] * b[2] + a[7] * b[5] + a[8] * b[8];

    m
}

fn get_screen_info(display: &XDisplayHandle) -> (i32, i32) {
    unsafe {
        let screen = XDefaultScreen(display.sys.as_ptr());
        let root = XRootWindow(display.sys.as_ptr(), screen);

        if (XRRGetScreenSizeRange(display.sys.as_ptr(), root, &mut 0, &mut 0, &mut 0, &mut 0))
            == True
        {
            let sw = XDisplayWidth(display.sys.as_ptr(), screen);
            let sh = XDisplayHeight(display.sys.as_ptr(), screen);

            return (sw, sh);
        } else {
            panic!("unable to determine screen size"); // TODO::loop through all active outputs + manually compute screen size
        }
    }
}

// fn map_to_output(input: &TouchInputIdentifier, input_display: &Output, output_display: &Output) {}

fn enumerate_touch_devices(display: &XDisplayHandle) -> Vec<TouchInputIdentifier> {
    let blacklist = ["Virtual core pointer"];

    let mut out = vec![];

    unsafe {
        let mut n_devices: i32 = 0;

        let info = XIQueryDevice(display.sys.as_ptr(), XIAllDevices, &mut n_devices);

        for i in 0..n_devices {
            let dev = info.offset(i as isize);
            for j in 0..(*dev).num_classes {
                let touch = (*dev).classes.offset(j as isize).read() as *mut XITouchClassInfo;
                if (*touch)._type == XITouchClass && (*touch).mode == XIDirectTouch {
                    let name = CStr::from_ptr((*dev).name).to_string_lossy().into_owned();
                    let deviceid = (*dev).deviceid;

                    if !blacklist.iter().any(|v| name == *v) {
                        out.push(TouchInputIdentifier { name, deviceid });
                    }
                }
            }
        }

        XIFreeDeviceInfo(info);
    }

    out
}

fn apply_matrix(display: &XDisplayHandle, deviceid: i32, m: &[f32; 9]) {
    unsafe {
        // Get the current property values

        let float_atom_name = CString::new("FLOAT").unwrap();
        let matrix_atom_name = CString::new("Coordinate Transformation Matrix").unwrap();

        let prop_float = XInternAtom(display.sys.as_ptr(), float_atom_name.as_ptr(), False);
        let prop_matrix = XInternAtom(display.sys.as_ptr(), matrix_atom_name.as_ptr(), False);

        if prop_float == 0 {
            panic!("let atom not found. This server is too old.");
        }
        if prop_matrix == 0 {
            panic!("Coordinate transformation matrix not found. This server is too old.");
        }

        let mut type_return: Atom = 0;
        let mut format_return: i32 = 0;
        let mut nitems: u64 = 0;
        let mut bytes_after: u64 = 0;
        let mut raw_data: *mut u8 = ptr::null_mut();

        let rc = XIGetProperty(
            display.sys.as_ptr(),
            deviceid,
            prop_matrix,
            0,
            9,
            False,
            prop_float,
            &mut type_return,
            &mut format_return,
            &mut nitems,
            &mut bytes_after,
            &mut raw_data,
        );

        if rc != Success as i32
            || prop_float != type_return
            || format_return != 32
            || nitems != 9
            || bytes_after != 0
        {
            panic!("Failed to retrieve current property values");
        }

        // Modify the retrieved property with the new matrix values
        let data = std::slice::from_raw_parts_mut(raw_data as *mut f32, nitems as usize);
        for i in 0..9 {
            data[i] = m[i];
        }

        // Apply the new property values
        XIChangeProperty(
            display.sys.as_ptr(),
            deviceid,
            prop_matrix,
            prop_float,
            format_return,
            PropModeReplace,
            raw_data,
            nitems as i32,
        );

        XFree(raw_data as *mut _);
    }
}
