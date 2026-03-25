import { readonly, ref } from "vue";

const visible = ref(false);
const title = ref("");
const message = ref("");
let resolve: ((v: boolean) => void) | null = null;

export function useConfirmDialog() {
  function ask(t: string, msg: string): Promise<boolean> {
    title.value = t;
    message.value = msg;
    visible.value = true;
    return new Promise((r) => {
      resolve = r;
    });
  }

  function confirm() {
    visible.value = false;
    resolve?.(true);
  }

  function cancel() {
    visible.value = false;
    resolve?.(false);
  }

  return {
    visible: readonly(visible),
    title: readonly(title),
    message: readonly(message),
    ask,
    confirm,
    cancel,
  };
}
