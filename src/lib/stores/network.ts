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

function loadCachedGeo(): GeoLocation | null {
  if (typeof localStorage !== 'undefined') {
    try {
      const saved = localStorage.getItem(STORAGE_KEY_GEO);
      if (saved) return JSON.parse(saved);
    } catch {}
  }
  return null;
}

const initialGeo = loadCachedGeo();

const initialState: NetworkState = {
  online: true,
  localIp: null,
  interfaceName: null,
  publicIp: initialGeo?.ip ?? null,
  pingMs: null,
  geo: initialGeo,
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

export async function refreshGeo() {
  if (typeof fetch === 'undefined') return;

  // Try ipwho.is first
  try {
    const res = await fetch('https://ipwho.is/', { signal: AbortSignal.timeout(4000) });
    if (res.ok) {
      const data = await res.json();
      if (data.success !== false && data.latitude && data.longitude) {
        const geo: GeoLocation = {
          ip: data.ip,
          city: data.city || '',
          region: data.region || '',
          country: data.country || '',
          countryCode: data.country_code || '',
          lat: Number(data.latitude),
          lon: Number(data.longitude),
          isp: data.connection?.isp || data.isp || '',
        };
        if (typeof localStorage !== 'undefined') {
          try {
            localStorage.setItem(STORAGE_KEY_GEO, JSON.stringify(geo));
          } catch {}
        }
        networkState.update((s) => ({
          ...s,
          geo,
          publicIp: geo.ip,
        }));
        return geo;
      }
    }
  } catch {}

  // Fallback to ip-api.com
  try {
    const res = await fetch('http://ip-api.com/json/', { signal: AbortSignal.timeout(4000) });
    if (res.ok) {
      const data = await res.json();
      if (data.status === 'success' && data.lat && data.lon) {
        const geo: GeoLocation = {
          ip: data.query,
          city: data.city || '',
          region: data.regionName || '',
          country: data.country || '',
          countryCode: data.countryCode || '',
          lat: Number(data.lat),
          lon: Number(data.lon),
          isp: data.isp || '',
        };
        if (typeof localStorage !== 'undefined') {
          try {
            localStorage.setItem(STORAGE_KEY_GEO, JSON.stringify(geo));
          } catch {}
        }
        networkState.update((s) => ({
          ...s,
          geo,
          publicIp: geo.ip,
        }));
        return geo;
      }
    }
  } catch {}

  return null;
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

  // Fetch geolocation if needed
  refreshGeo().catch(() => {});

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
