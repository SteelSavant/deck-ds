import pathlib
import subprocess
import asyncio
import os
import decky # type: ignore

HOME_DIR = str(pathlib.Path(os.getcwd()).parent.parent.resolve())
PARENT_DIR = str(pathlib.Path(__file__).parent.resolve())

logger = decky.logger

class Plugin:
    backend_proc = None

    async def plugin_info(self):
        # Taken from https://github.com/jurassicplayer/decky-autosuspend/blob/main/main.py, BSD-3 Clause License

        # Call plugin_info only once preferably
        logger.debug('[backend] PluginInfo:\n\tPluginName: {}\n\tPluginVersion: {}\n\tDeckyVersion: {}'.format(
            decky.DECKY_PLUGIN_NAME,
            decky.DECKY_PLUGIN_VERSION,
            decky.DECKY_VERSION
        ))
        pluginInfo = {
            "name": decky.DECKY_PLUGIN_NAME,
            "version": decky.DECKY_PLUGIN_VERSION
        }
        return pluginInfo
    
    # Asyncio-compatible long-running code, executed in a task when the plugin is loaded
    async def _main(self):
        # startup
        logger.info("DeckDS starting...")
        env_proc = dict(os.environ)
        if "LD_LIBRARY_PATH" in env_proc:
            env_proc["LD_LIBRARY_PATH"] += ":"+PARENT_DIR+"/bin"
        else:
            env_proc["LD_LIBRARY_PATH"] = ":"+PARENT_DIR+"/bin"
        self.backend_proc = subprocess.Popen(
            [PARENT_DIR + "/bin/backend"],
            env = env_proc)
        while True:
            await asyncio.sleep(1)

    async def _unload(self):
        # shutdown
        logger.info("DeckDS unloading...")
        if self.backend_proc is not None:
            self.backend_proc.terminate()
            try:
                self.backend_proc.wait(timeout=5) # 5 seconds timeout
            except subprocess.TimeoutExpired:
                self.backend_proc.kill()
            self.backend_proc = None
