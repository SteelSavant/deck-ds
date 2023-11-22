import { ApiError } from "../backend";
import { Result } from "./result";

export type Loading<T> = Result<T, ApiError> | null | undefined;
