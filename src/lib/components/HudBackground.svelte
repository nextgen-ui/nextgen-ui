<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { currentTheme } from '$lib/stores/theme';

  /**
   * Decorative HUD backdrop.
   *
   * This runs forever behind the whole app, so it is written to cost as little
   * as possible: gradients are built once and reused, per-line rgba strings are
   * replaced by globalAlpha over a solid stroke, grid lines are batched into a
   * handful of strokes by quantised alpha, and the loop is capped well below
   * display refresh — dropping to a crawl when the window loses focus and
   * stopping outright when the page is hidden.
   */

  const COLS = 36;
  const ROWS = 22;
  const MAX_P = 50;

  const ACTIVE_FPS = 30;   // foreground: smooth enough for a slow pulse
  const IDLE_FPS = 5;      // unfocused: keep it alive but nearly free
  const ALPHA_BUCKETS = 6; // grid lines batched into this many strokes

  let canvas: HTMLCanvasElement;
  let animFrame = 0;
  let theme = $currentTheme;
  let gradsDirty = true;

  const unsubTheme = currentTheme.subscribe((t) => { theme = t; gradsDirty = true; });

  onMount(() => {
    const ctx2d = canvas.getContext('2d');
    if (!ctx2d) return;
    const ctx: CanvasRenderingContext2D = ctx2d;

    let W = canvas.width = window.innerWidth;
    let H = canvas.height = window.innerHeight;
    let cellW = W / COLS;
    let cellH = H / ROWS;
    let t = 0;

    let rgb = '0,200,255';
    let solid = `rgb(${rgb})`;
    let cornerGrads: CanvasGradient[] = [];
    let scanGrad: CanvasGradient | null = null;

    // Reused every frame so the pulse is computed once per line, not once per
    // line per alpha bucket.
    const colPulse = new Float32Array(COLS + 1);
    const rowPulse = new Float32Array(ROWS + 1);

    type Particle = { x: number; y: number; vx: number; vy: number; life: number; maxLife: number; size: number };
    const particles: Particle[] = [];
    type Drop = { x: number; y: number; char: string; alpha: number };
    const drops: Drop[] = [];

    function spawn() {
      particles.push({
        x: Math.random() * W, y: Math.random() * H,
        vx: (Math.random() - 0.5) * 0.25,
        vy: (Math.random() - 0.5) * 0.25,
        life: 0,
        maxLife: 100 + Math.random() * 150,
        size: 1 + Math.random() * 1.5,
      });
    }
    for (let i = 0; i < MAX_P; i++) spawn();

    /** Gradients depend only on size and theme, so build them only then. */
    function rebuildGradients() {
      const accent = (theme.accent || '#00c8ff').replace('#', '');
      rgb = `${parseInt(accent.slice(0, 2), 16)},${parseInt(accent.slice(2, 4), 16)},${parseInt(accent.slice(4, 6), 16)}`;
      solid = `rgb(${rgb})`;

      cornerGrads = ([[0, 0], [W, 0], [0, H], [W, H]] as [number, number][]).map(([cx, cy]) => {
        const g = ctx.createRadialGradient(cx, cy, 0, cx, cy, 180);
        g.addColorStop(0, `rgba(${rgb},0.07)`);
        g.addColorStop(1, `rgba(${rgb},0)`);
        return g;
      });

      // Built in local space (0..30) and positioned by translating the canvas,
      // so the moving scan line does not need a new gradient every frame.
      scanGrad = ctx.createLinearGradient(0, 0, 0, 30);
      scanGrad.addColorStop(0, `rgba(${rgb},0)`);
      scanGrad.addColorStop(0.5, `rgba(${rgb},0.05)`);
      scanGrad.addColorStop(1, `rgba(${rgb},0)`);

      ctx.font = '10px JetBrains Mono, monospace';
      gradsDirty = false;
    }

    function draw() {
      if (gradsDirty) rebuildGradients();
      t++;
      ctx.clearRect(0, 0, W, H);

      for (let i = 0; i <= COLS; i++) colPulse[i] = Math.sin(t * 0.02 + i * 0.3) * 0.5 + 0.5;
      for (let j = 0; j <= ROWS; j++) rowPulse[j] = Math.sin(t * 0.015 + j * 0.4) * 0.5 + 0.5;

      // Grid — one stroke per alpha bucket instead of one per line.
      ctx.lineWidth = 0.5;
      ctx.strokeStyle = solid;
      for (let b = 0; b < ALPHA_BUCKETS; b++) {
        ctx.globalAlpha = 0.025 + ((b + 0.5) / ALPHA_BUCKETS) * 0.025;
        ctx.beginPath();
        for (let i = 0; i <= COLS; i++) {
          if (Math.min(ALPHA_BUCKETS - 1, (colPulse[i] * ALPHA_BUCKETS) | 0) !== b) continue;
          const x = i * cellW;
          ctx.moveTo(x, 0); ctx.lineTo(x, H);
        }
        for (let j = 0; j <= ROWS; j++) {
          if (Math.min(ALPHA_BUCKETS - 1, (rowPulse[j] * ALPHA_BUCKETS) | 0) !== b) continue;
          const y = j * cellH;
          ctx.moveTo(0, y); ctx.lineTo(W, y);
        }
        ctx.stroke();
      }
      ctx.globalAlpha = 1;

      // Scan line
      const scanY = (t * 0.7 * (60 / ACTIVE_FPS)) % (H + 40) - 20;
      if (scanGrad) {
        ctx.save();
        ctx.translate(0, scanY - 15);
        ctx.fillStyle = scanGrad;
        ctx.fillRect(0, 0, W, 30);
        ctx.restore();
      }

      // Corner glow
      const corners: [number, number][] = [[0, 0], [W, 0], [0, H], [W, H]];
      for (let i = 0; i < cornerGrads.length; i++) {
        const [cx, cy] = corners[i];
        ctx.fillStyle = cornerGrads[i];
        ctx.fillRect(cx === 0 ? 0 : cx - 180, cy === 0 ? 0 : cy - 180, 180, 180);
      }

      // Particles — one fillStyle, alpha varied per particle.
      ctx.fillStyle = solid;
      for (let i = particles.length - 1; i >= 0; i--) {
        const p = particles[i];
        p.x += p.vx; p.y += p.vy; p.life++;
        const prog = p.life / p.maxLife;
        const a = prog < 0.2 ? (prog / 0.2) * 0.4 : prog > 0.8 ? ((1 - prog) / 0.2) * 0.4 : 0.4;
        ctx.globalAlpha = a * 0.5;
        ctx.beginPath();
        ctx.arc(p.x, p.y, p.size, 0, Math.PI * 2);
        ctx.fill();
        if (p.life >= p.maxLife) { particles.splice(i, 1); spawn(); }
      }

      // Data drops
      if (t % 2 === 0 && Math.random() < 0.25) {
        drops.push({
          x: Math.floor(Math.random() * COLS) * cellW + cellW / 2,
          y: Math.random() * H,
          char: Math.random() > 0.5 ? '1' : '0',
          alpha: 0.2,
        });
      }
      for (let i = drops.length - 1; i >= 0; i--) {
        const dr = drops[i];
        ctx.globalAlpha = dr.alpha;
        ctx.fillText(dr.char, dr.x, dr.y);
        dr.alpha -= 0.016;
        if (dr.alpha <= 0) drops.splice(i, 1);
      }
      ctx.globalAlpha = 1;
    }

    let last = 0;
    function loop(now: number) {
      animFrame = requestAnimationFrame(loop);
      const budget = 1000 / (document.hasFocus() ? ACTIVE_FPS : IDLE_FPS);
      if (now - last < budget) return;
      last = now;
      draw();
    }

    function start() {
      if (!animFrame) animFrame = requestAnimationFrame(loop);
    }
    function stop() {
      if (animFrame) { cancelAnimationFrame(animFrame); animFrame = 0; }
    }

    // Nothing is visible when the page is hidden, so do not paint at all.
    const onVisibility = () => (document.hidden ? stop() : start());

    const onResize = () => {
      W = canvas.width = window.innerWidth;
      H = canvas.height = window.innerHeight;
      cellW = W / COLS;
      cellH = H / ROWS;
      gradsDirty = true;
    };

    start();
    window.addEventListener('resize', onResize);
    document.addEventListener('visibilitychange', onVisibility);

    return () => {
      stop();
      window.removeEventListener('resize', onResize);
      document.removeEventListener('visibilitychange', onVisibility);
    };
  });

  onDestroy(() => {
    if (animFrame) cancelAnimationFrame(animFrame);
    unsubTheme();
  });
</script>

<canvas bind:this={canvas} class="hud-bg"></canvas>

<style>
  .hud-bg {
    position: fixed; inset: 0;
    width: 100vw; height: 100vh;
    pointer-events: none; z-index: 0;
  }
</style>
