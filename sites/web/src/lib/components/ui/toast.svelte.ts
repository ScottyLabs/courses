// App-wide toast notifications. Call `toast("Saved")` from anywhere; the root
// layout renders <Toaster />.

export type ToastKind = "info" | "success" | "error";

export interface ToastAction {
  label: string;
  run: () => void;
}

export interface ToastItem {
  id: number;
  message: string;
  kind: ToastKind;
  action?: ToastAction;
  /** Milliseconds before it hides itself; 0 keeps it until dismissed. */
  duration: number;
}

export interface ToastOptions {
  kind?: ToastKind;
  action?: ToastAction;
  duration?: number;
}

let nextId = 1;

class ToastQueue {
  items = $state<ToastItem[]>([]);

  push(message: string, options: ToastOptions = {}): number {
    const kind = options.kind ?? "info";
    const item: ToastItem = {
      id: nextId,
      message,
      kind,
      action: options.action,
      duration: options.duration ?? (kind === "error" ? 8000 : 5000),
    };
    nextId += 1;
    this.items = [...this.items.slice(-3), item];
    return item.id;
  }

  dismiss(id: number): void {
    this.items = this.items.filter((item) => item.id !== id);
  }
}

export const toasts = new ToastQueue();

/** Shows a toast and returns its id (for `dismissToast`). */
export function toast(message: string, options?: ToastOptions): number {
  return toasts.push(message, options);
}

export function dismissToast(id: number): void {
  toasts.dismiss(id);
}
