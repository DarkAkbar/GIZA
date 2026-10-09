export const OPERATIONS_REFRESH_EVENT = "giza.operations.refresh";

export function notifyOperationsChanged() {
  window.dispatchEvent(new CustomEvent(OPERATIONS_REFRESH_EVENT));
}
