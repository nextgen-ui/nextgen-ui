/**
 * Equirectangular land/sea mask, 192x96 cells, one bit per cell, rows padded to
 * whole bytes and packed MSB first.
 *
 * Derived offline from Natural Earth 110m land polygons (public domain) via the
 * world-atlas TopoJSON build. Rasterised at generation time, so nothing is
 * fetched or parsed at runtime — this is ~2.3KB of bits rather than a
 * geometry library and a 55KB topology.
 */
export const LAND_W = 192;
export const LAND_H = 96;

const LAND_MASK_B64 =
  'AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAP/8B//gAAAAAAAAAAAAAAAAAAAAAAAAH///////AAAgA8AAAOAAAAAAAAAAAAAn///////8AB/4B8AAA/gAAAAAAAAAAAP3///////4AAfwAAAgAB8AAAAAAAAAAA/9//j////wAAMAAB/gD//gAf4AAAAAAB////gB///wAAAAADwA///9wOwAAwAgAD////8A///gAAAAAHA/////7/4AAwH/8//////A///gAAD/ADt//////////wf//////+/wf/+AAAf/9////////////////////+/8f/B+AA///////////////+f//////+/wP4B+AB///////////////j///////f/wP4AYAP///////////////A//////8B+QHwAAAP//////////////8Af/P///+A/4AAAAGP8///////////+egAD4A////w/8AAAAOH5//////////4B8AAPAAf//////AAAAfD///////////gD4AAAAA///////gAAA/v///////////+BwAAAAAf//////gAAA/////////////+BgAAAAAH//////wAAAP/////////////AAAAAAAB/////3wAAAH////////////+AAAAAAAB/////+QAAAB////////////2AAAAAAAB/////4AAAAf//9v////////vAAAAAAAB////+AAAAA/z/93////////OAAAAAAAB////8AAAAA/zf/////////8MAAAAAAAA////4AAAAA/j/v////////4MAAAAAAAAf///wAAAAAf/Zv////////d8AAAAAAAAf///wAAAAAP/A5///////+f4AAAAAAAAP///AAAAAA//7w///////+HgAAAAAAAAH//+AAAAAA////////////GAAAAAAAAAD//vAAAAAB////////////AAAAAAAAAAD/4HgAAAAD///////////+AAAAAAAAAAA/4DgAAAAH/////9//////AAAAAAAAAAA/wHgAAAAH//////x////6AAAAAADgAAAH5/4AAAAP//////w////iAAAAAAAwAAAH5w/AAAAH////3/wP+f/AAAAAAAAAAAAD/w/wAAAH//////gP4P+DAAAAAAAAAAAA/8AAAAAH/////+AHwP+HAAAAAAAAAAAAD8AAAAAP/////4AHgD+DgAAAAAAAAAAAAcPAAAAH/////8AHgD+HwAAAAAAAAAAAAN/8AAAH/////8ADgD8FwAAAAAAAAAAAAH/+AAAD/////4ADwDYNwAAAAAAAAAAAAD//wAAB/////4AAwBgdwAAAAAAAAAAAAA//4AAA/P///wAAAPw8gAAAAAAAAAAAAA//8AAAAB///gAAAHz4IAAAAAAAAAAAAB//8AAAAD///AAAADz/4AAAAAAAAAAAAD///gAAAD//+AAAABz/PQAAAAAAAAAAAD///8AAAB//8AAAAB7/f+MAAAAAAAAAAD////AAAB//8AAAAA4XX/eAAAAAAAAAAD////AAAA//8AAAAAflBf7gAAAAAAAAAB////AAAA//8AAAAAH/wfw4AAAAAAAAAA///+AAAA//8AAAAAAHiG4YAAAAAAAAAA///8AAAAf/8IAAAAAAH2AAAAAAAAAAAAf//8AAAA//8YAAAAAA/nADAgAAAAAAAAf//8AAAA//94AAAAAB/3ADBgAAAAAAAAH//8AAAA//54AAAAAD//AADAAAAAAAAAD//4AAAA//x4AAAAAP//wGAAAAAAAAAAD//4AAAAf/hwAAAAA///4HAAAAAAAAAAD//wAAAAf/hwAAAAA///4AAAAAAAAAAAD/+AAAAAf/hwAAAAA///8AAAAAAAAAAAD/+AAAAAP/AAAAAAA///8AAAAAAAAAAAH/+AAAAAP/AAAAAAAf//8AAAAAAAAAAAH/4AAAAAH+AAAAAAAf//8AAAAAAAAAAAH/4AAAAAH8AAAAAAAf7/8AAAAAAAAAAAH/wAAAAAHwAAAAAAAfA/4AIAAAAAAAAAP/AAAAAAAAAAAAAAAAAP4AMgAAAAAAAAP/AAAAAAAAAAAAAAAAAHwAPAAAAAAAAAP4AAAAAAAAAAAAAAAAABwAeAAAAAAAAAP4AAAAAAAAAAAAAAAAABwA8AAAAAAAAAPwAAAAAAAAAAAAAAAAABgB4AAAAAAAAAfwAAAAAAAAAAAAAAAAAAADgAAAAAAAAAfgAAAAAAAAAAAgAAAAAAAAAAAAAAAAAAfAAAAAAAAAAAAwAAAAAAAAAAAAAAAAAAfHAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAPAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAHwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAADAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAfAAAAAAAAAAAAAAgAAgAAAAAAAAAAAAB4AAAAAAAAAfyAf/////AAAAAAAAAAAAD4AAAAAAAB3//x///////AAAAAAAAAAAP8AAAAJf//////////////gAAAAAAD4of8AAAD////////////////wAAAH//7///8AAAH////////////////gAAB///////wAAP////////////////8AAP///////4AHj/////////////////+AAHf//////4+fj//////////////////AAA////////+f//////////////////8A/gP////////////////////////////0////////////////////////////////AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA';

let mask: Uint8Array | null = null;

/** Decode once, lazily — callers sample it while building the point cloud. */
function bits(): Uint8Array {
  if (mask) return mask;
  const bin = atob(LAND_MASK_B64);
  const out = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
  mask = out;
  return out;
}

const ROW_BYTES = (LAND_W + 7) >> 3;

/** True when the cell at column x, row y (from the north pole) is land. */
export function isLand(x: number, y: number): boolean {
  if (x < 0 || x >= LAND_W || y < 0 || y >= LAND_H) return false;
  const b = bits()[y * ROW_BYTES + (x >> 3)];
  return (b & (0x80 >> (x & 7))) !== 0;
}

/** Sample by geographic coordinate. */
export function isLandAt(lonDeg: number, latDeg: number): boolean {
  const x = Math.floor(((lonDeg + 180) / 360) * LAND_W);
  const y = Math.floor(((90 - latDeg) / 180) * LAND_H);
  return isLand(x, y);
}
