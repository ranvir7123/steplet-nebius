// A question that covers the window: a <dialog> opened as modal, so nothing behind it can be clicked or reached with
// the keyboard until it is answered. Escape does not answer it; the control marked data-first gets the focus.
export function modal(node: HTMLDialogElement) {
  const cancel = (e: Event) => e.preventDefault();
  node.addEventListener("cancel", cancel);
  node.showModal();
  node.querySelector<HTMLElement>("[data-first]")?.focus();
  return {
    destroy() {
      node.removeEventListener("cancel", cancel);
      if (node.open) node.close();
    },
  };
}
