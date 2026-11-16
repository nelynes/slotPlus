const GBA_CART = 360 / 212
const GB_CART = 312 / 337

// slotPlus's carts, at the sizes slotPlus draws them.
const CART_ASPECTS: Record<string, number> = {
  WS: 352 / 211,
  Lynx: 312 / 328,
  PokeMini: 300 / 262,
}

export function cartAspect(tag: string): number {
  return CART_ASPECTS[tag] ?? (tag === 'GB' || tag === 'GBC' ? GB_CART : GBA_CART)
}

export const MIN_PAGE_SIZE = 20

export function pageSizeFor(cols: number, min: number = MIN_PAGE_SIZE): number {
  const c = Math.max(1, Math.floor(cols))
  const floor = Math.max(1, Math.floor(min))
  return c * Math.ceil(floor / c)
}
