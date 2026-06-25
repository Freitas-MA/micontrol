/**
 * Error response from the Rust backend.
 * Matches the `ErrorResponse` struct in `src-tauri/src/hw/errors.rs`.
 */
export interface ErrorResponse {
  code: string;
  message: string;
}

/**
 * Stable error codes matching the Rust `HardwareError::code()` method.
 * @see src-tauri/src/hw/errors.rs
 */
export const ERROR_CODES = {
  WMI_QUERY: 'WMI_QUERY',
  WMI_CONNECTION: 'WMI_CONNECTION',
  REGISTRY: 'REGISTRY',
  DEVICE_NOT_FOUND: 'DEVICE_NOT_FOUND',
  PERMISSION_DENIED: 'PERMISSION_DENIED',
  INVALID_ARGUMENT: 'INVALID_ARGUMENT',
  IO: 'IO',
  TIMEOUT: 'TIMEOUT',
  CRYPTO: 'CRYPTO',
  IPC: 'IPC',
  HARDWARE: 'HARDWARE',
  AI_CONSENT_DENIED: 'AI_CONSENT_DENIED',
  AI_REQUEST_FAILED: 'AI_REQUEST_FAILED',
  AI_RESPONSE_INVALID: 'AI_RESPONSE_INVALID',
  GENERIC: 'GENERIC',
} as const;

/**
 * Parse an error from a Tauri invoke rejection.
 * Tauri rejects with a string (the error message) or an object with code+message.
 */
export function parseErrorResponse(error: unknown): ErrorResponse {
  if (typeof error === 'string') {
    // Try to parse as JSON (ErrorResponse format)
    try {
      const parsed = JSON.parse(error);
      if (parsed.code && parsed.message) {
        return { code: parsed.code, message: parsed.message };
      }
    } catch {
      // Not JSON, treat as generic error
    }
    return { code: ERROR_CODES.GENERIC, message: error };
  }
  if (error && typeof error === 'object' && 'code' in error && 'message' in error) {
    return {
      code: String((error as ErrorResponse).code),
      message: String((error as ErrorResponse).message),
    };
  }
  return { code: ERROR_CODES.GENERIC, message: 'An unexpected error occurred' };
}

/**
 * Translation function type — matches the `t` function from `useI18n`.
 * Accepts a dot-path key and returns the localized string.
 */
export type TranslateFn = (key: string) => string;

/**
 * Get a user-friendly error message based on the error code.
 *
 * @param error  The parsed error response from the backend.
 * @param t      Translation function from `useI18n`. When omitted, falls back
 *               to the raw English strings (useful for non-React contexts).
 */
export function getUserFriendlyMessage(error: ErrorResponse, t?: TranslateFn): string {
  const tr = t ?? ((key: string) => FALLBACK_MESSAGES[key] ?? key);
  switch (error.code) {
    case ERROR_CODES.WMI_QUERY:
    case ERROR_CODES.WMI_CONNECTION:
      return tr('errors.wmiUnavailable');
    case ERROR_CODES.DEVICE_NOT_FOUND:
      return tr('errors.deviceNotFound');
    case ERROR_CODES.PERMISSION_DENIED:
      return tr('errors.permissionDeniedAdmin');
    case ERROR_CODES.TIMEOUT:
      return tr('errors.timeout');
    case ERROR_CODES.AI_CONSENT_DENIED:
      return tr('errors.aiConsentDenied');
    case ERROR_CODES.AI_REQUEST_FAILED:
      return tr('errors.aiRequestFailed');
    case ERROR_CODES.AI_RESPONSE_INVALID:
      return tr('errors.aiResponseInvalid');
    default:
      return error.message || tr('errors.unexpected');
  }
}

/** English fallback strings used when no translation function is provided. */
const FALLBACK_MESSAGES: Record<string, string> = {
  'errors.wmiUnavailable':
    'Hardware information is temporarily unavailable. The system may be busy.',
  'errors.deviceNotFound': 'The requested hardware device was not found.',
  'errors.permissionDeniedAdmin': 'Permission denied. Administrator privileges may be required.',
  'errors.timeout': 'The operation timed out. Please try again.',
  'errors.aiConsentDenied':
    'AI analysis requires telemetry consent. Please grant consent in Settings.',
  'errors.aiRequestFailed': 'AI analysis failed. Please check your connection and try again.',
  'errors.aiResponseInvalid': 'AI returned an invalid response. Please try again.',
  'errors.unexpected': 'An unexpected error occurred.',
};
