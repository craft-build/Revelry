interface Toast {
  id: string;
  message: string;
  variant: "success" | "error";
}

class ToastStore {
  list = $state<Toast[]>([]);

  show(message: string, variant: Toast["variant"] = "success") {
    const id = crypto.randomUUID();
    this.list = [...this.list, { id, message, variant }];
    setTimeout(() => this.dismiss(id), 4000);
  }

  dismiss(id: string) {
    this.list = this.list.filter((t) => t.id !== id);
  }
}

export const toasts = new ToastStore();
