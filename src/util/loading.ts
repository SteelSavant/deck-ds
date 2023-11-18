import { ApiError } from "../backend";
import { Option } from "./option";
import { Result } from "./result";

export type Loading<T> = Option<Result<T, ApiError>>;
export { None, Some } from "./option";
