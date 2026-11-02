import type { LocationQueryRaw } from 'vue-router'

export const PLATFORM_TAGS = [
  'GB', 'GBC', 'GBA', 'GBAV', 'WS', 'Lynx', 'GG', 'NGP', 'TG16', 'NDS',
] as const

export const PLATFORM_NAMES: Record<string, string> = {
  GBA: 'Game Boy Advance',
  GBAV: 'Game Boy Advance Video',
  GB: 'Game Boy',
  GBC: 'Game Boy Color',
  WS: 'WonderSwan',
  Lynx: 'Atari Lynx',
  GG: 'Game Gear',
  NGP: 'NeoGeo Pocket',
  TG16: 'TurboGrafx-16',
  NDS: 'Nintendo DS',
}

export const HOME_TAGS = ['GB', 'GBC', 'GBA', 'WS', 'Lynx', 'GG', 'NGP', 'TG16', 'NDS'] as const

export const VIDEO_TAG = 'GBAV'
export const VIDEO_HOME = 'GBA'

export function platformRoute(tag: string, query: LocationQueryRaw = {}) {
  return tag === VIDEO_TAG
    ? { name: 'platform-video', params: { tag: VIDEO_HOME }, query }
    : { name: 'platform', params: { tag }, query }
}

// slotPlus's systems: the Lynx needs its boot ROM; the others boot without one (the DS's
// own dumps are optional and go in BIOS/ by hand).
const BIOS_FILES: Record<string, string> = {
  Lynx: 'lynxboot.img',
}

export const NO_BIOS: readonly string[] = ['WS', 'GG', 'NGP', 'TG16', 'NDS']

export function biosFile(tag: string): string {
  return BIOS_FILES[tag] ?? `${tag.toLowerCase()}_bios.bin`
}

export function platformIcon(tag: string): string {
  return `/platforms/${tag}.svg`
}

export function platformName(tag: string): string {
  return PLATFORM_NAMES[tag] ?? tag
}
