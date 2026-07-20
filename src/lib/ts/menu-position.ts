export function clampMenu(node: HTMLElement) {
  const margin = 8;
  const rect = node.getBoundingClientRect();

  let left = rect.left;
  let top = rect.top;

  if (rect.right > window.innerWidth - margin) {
    left = window.innerWidth - rect.width - margin;
  }
  if (rect.bottom > window.innerHeight - margin) {
    top = window.innerHeight - rect.height - margin;
  }

  left = Math.max(margin, left);
  top = Math.max(margin, top);

  node.style.left = `${left}px`;
  node.style.top = `${top}px`;
}
