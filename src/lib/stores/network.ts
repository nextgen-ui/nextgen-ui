import { writable } from 'svelte/store';
import { getLocalNetInfo, measurePing } from '../ipc/index.ts';

export interface GeoLocation {
  ip: string;
  city: string;
  region: string;
  country: string;
  countryCode: string;
  lat: number;
  lon: number;
  isp?: string;
}

export interface NetworkState {
  online: boolean;
  localIp: string | null;
  interfaceName: string | null;
  publicIp: string | null;
  pingMs: number | null;
  geo: GeoLocation | null;
  lastPingAt: number;
}

const STORAGE_KEY_GEO = 'nextgen_ui_geo';

// Geolocation is opt-in (config `geoLookup`). While it is off, nothing is sent
// to a third party, and no public IP or location is held, cached or displayed.
let geoEnabled = false;

function loadCachedGeo(): GeoLocation | null {
  if (typeof localStorage !== 'undefined') {
    try {
      const saved = localStorage.getItem(STORAGE_KEY_GEO);
      if (saved) return JSON.parse(saved);
    } catch {}
  }
  return null;
}

function clearCachedGeo() {
  if (typeof localStorage !== 'undefined') {
    try {
      localStorage.removeItem(STORAGE_KEY_GEO);
    } catch {}
  }
}

const initialState: NetworkState = {
  online: true,
  localIp: null,
  interfaceName: null,
  publicIp: null,
  pingMs: null,
  geo: null,
  lastPingAt: 0,
};

export const networkState = writable<NetworkState>(initialState);

let pingTimer: ReturnType<typeof setInterval> | null = null;
let initialized = false;

export async function refreshPing() {
  try {
    const ping = await measurePing();
    networkState.update((s) => ({
      ...s,
      pingMs: ping,
      online: ping !== null,
      lastPingAt: Date.now(),
    }));
    return ping;
  } catch {
    networkState.update((s) => ({ ...s, pingMs: null, online: false }));
    return null;
  }
}

/** Turn geolocation on or off. Turning it off also forgets what was fetched. */
export function setGeoEnabled(enabled: boolean) {
  geoEnabled = enabled;
  if (enabled) {
    const cached = loadCachedGeo();
    if (cached) networkState.update((s) => ({ ...s, geo: cached, publicIp: cached.ip }));
    refreshGeo().catch(() => {});
  } else {
    clearCachedGeo();
    networkState.update((s) => ({ ...s, geo: null, publicIp: null }));
  }
}

export function isGeoEnabled() {
  return geoEnabled;
}

export async function refreshGeo(): Promise<GeoLocation | null> {
  if (!geoEnabled || typeof fetch === 'undefined') return null;

  // HTTPS only. The former fallback, ip-api.com, is plain HTTP on its free
  // tier, so anyone on the network could read the lookup or forge the answer.
  try {
    const res = await fetch('https://ipwho.is/', { signal: AbortSignal.timeout(4000) });
    if (!res.ok) return null;
    const data = await res.json();
    const lat = Number(data.latitude);
    const lon = Number(data.longitude);
    if (data.success === false || !Number.isFinite(lat) || !Number.isFinite(lon)) return null;
    // Disabled while the request was in flight: drop the answer.
    if (!geoEnabled) return null;

    const geo: GeoLocation = {
      ip: String(data.ip ?? ''),
      city: String(data.city ?? ''),
      region: String(data.region ?? ''),
      country: String(data.country ?? ''),
      countryCode: String(data.country_code ?? ''),
      lat,
      lon,
      isp: String(data.connection?.isp ?? data.isp ?? ''),
    };
    if (typeof localStorage !== 'undefined') {
      try {
        localStorage.setItem(STORAGE_KEY_GEO, JSON.stringify(geo));
      } catch {}
    }
    networkState.update((s) => ({ ...s, geo, publicIp: geo.ip }));
    return geo;
  } catch {
    return null;
  }
}

export function initNetworkService() {
  if (initialized || typeof window === 'undefined') return;
  initialized = true;

  // Fetch local network info immediately
  getLocalNetInfo()
    .then((info) => {
      networkState.update((s) => ({
        ...s,
        localIp: info.local_ip,
        interfaceName: info.interface_name,
        pingMs: info.ping_ms ?? s.pingMs,
        online: info.ping_ms !== null,
      }));
    })
    .catch(() => {});

  // Geolocation is not started here — the page calls setGeoEnabled() once
  // the config has loaded, and only if the user opted in.

  // Set up periodic ping (every 4 seconds when focused, 15 seconds when idle)
  const runPing = () => {
    if (!document.hidden) {
      refreshPing().catch(() => {});
    }
  };

  pingTimer = setInterval(runPing, 3500);

  document.addEventListener('visibilitychange', () => {
    if (!document.hidden) {
      runPing();
    }
  });
}
