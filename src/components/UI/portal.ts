export function portal(element: HTMLElement, target: string | HTMLElement = 'body') {
  let targetElement: HTMLElement | null

  function update(newTarget: string | HTMLElement) {
    targetElement = typeof newTarget === 'string'
      ? document.querySelector(newTarget)
      : newTarget

    if (targetElement) {
      targetElement.appendChild(element)
    }
  }

  update(target)

  return {
    update,
    destroy() {
      if (element && element.parentNode) {
        element.parentNode.removeChild(element)
      }
    },
  }
}
