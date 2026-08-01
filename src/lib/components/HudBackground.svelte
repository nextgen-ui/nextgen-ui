<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { currentTheme } from '$lib/stores/theme';

  let canvas: HTMLCanvasElement;
  let animFrame: number;
  let theme = $currentTheme;
  const unsubTheme = currentTheme.subscribe((t) => { theme = t; });

  function hexToRgb(hex: string): [number, number, number] {
    const clean = hex.replace('#', '');
    const r = parseInt(clean.slice(0, 2), 16) / 255;
    const g = parseInt(clean.slice(2, 4), 16) / 255;
    const b = parseInt(clean.slice(4, 6), 16) / 255;
    return [r, g, b];
  }

  onMount(() => {
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    let W = canvas.width  = window.innerWidth;
    let H = canvas.height = window.innerHeight;
    let t = 0;

    // Grid
    const COLS = 36;
    const ROWS = 22;
    let cellW = W / COLS;
    let cellH = H / ROWS;

    // Particles
    type Particle = { x: number; y: number; vx: number; vy: number; life: number; maxLife: number; size: number };
    const particles: Particle[] = [];
    const MAX_P = 50;

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

    // Data drops
    type Drop = { x: number; y: number; char: string; alpha: number };
    const drops: Drop[] = [];

    function draw() {
      t++;
      ctx.clearRect(0, 0, W, H);

      const accent = (theme.accent || '#00c8ff').replace('#', '');
      const rr = parseInt(accent.slice(0,2), 16);
      const gg = parseInt(accent.slice(2,4), 16);
      const bb = parseInt(accent.slice(4,6), 16);
      const rgb = `${rr},${gg},${bb}`;

      // Grid lines
      ctx.lineWidth = 0.5;
      for (let i = 0; i <= COLS; i++) {
        const x = i * cellW;
        const pulse = Math.sin(t * 0.02 + i * 0.3) * 0.5 + 0.5;
        ctx.strokeStyle = `rgba(${rgb},${0.025 + pulse * 0.025})`;
        ctx.beginPath(); ctx.moveTo(x, 0); ctx.lineTo(x, H); ctx.stroke();
      }
      for (let j = 0; j <= ROWS; j++) {
        const y = j * cellH;
        const pulse = Math.sin(t * 0.015 + j * 0.4) * 0.5 + 0.5;
        ctx.strokeStyle = `rgba(${rgb},${0.025 + pulse * 0.025})`;
        ctx.beginPath(); ctx.moveTo(0, y); ctx.lineTo(W, y); ctx.stroke();
      }

      // Scan line
      const scanY = (t * 0.7) % (H + 40) - 20;
      const sg = ctx.createLinearGradient(0, scanY - 15, 0, scanY + 15);
      sg.addColorStop(0,   `rgba(${rgb},0)`);
      sg.addColorStop(0.5, `rgba(${rgb},0.05)`);
      sg.addColorStop(1,   `rgba(${rgb},0)`);
      ctx.fillStyle = sg;
      ctx.fillRect(0, scanY - 15, W, 30);

      // Corner glow
      for (const [cx, cy] of [[0,0],[W,0],[0,H],[W,H]] as [number,number][]) {
        const g2 = ctx.createRadialGradient(cx, cy, 0, cx, cy, 180);
        g2.addColorStop(0, `rgba(${rgb},0.07)`);
        g2.addColorStop(1, `rgba(${rgb},0)`);
        ctx.fillStyle = g2;
        ctx.fillRect(cx === 0 ? 0 : cx-180, cy === 0 ? 0 : cy-180, 180, 180);
      }

      // Particles
      for (let i = particles.length - 1; i >= 0; i--) {
        const p = particles[i];
        p.x += p.vx; p.y += p.vy; p.life++;
        const prog = p.life / p.maxLife;
        const alpha = prog < 0.2 ? (prog/0.2)*0.4 : prog > 0.8 ? ((1-prog)/0.2)*0.4 : 0.4;
        ctx.beginPath();
        ctx.arc(p.x, p.y, p.size, 0, Math.PI*2);
        ctx.fillStyle = `rgba(${rgb},${alpha*0.5})`;
        ctx.fill();
        if (p.life >= p.maxLife) { particles.splice(i,1); spawn(); }
      }

      // Data drops
      if (t % 4 === 0 && Math.random() < 0.25) {
        drops.push({
          x: Math.floor(Math.random() * COLS) * cellW + cellW/2,
          y: Math.random() * H,
          char: Math.random() > 0.5 ? '1' : '0',
          alpha: 0.2,
        });
      }
      ctx.font = '10px JetBrains Mono, monospace';
      for (let i = drops.length - 1; i >= 0; i--) {
        const d = drops[i];
        ctx.fillStyle = `rgba(${rgb},${d.alpha})`;
        ctx.fillText(d.char, d.x, d.y);
        d.alpha -= 0.008;
        if (d.alpha <= 0) drops.splice(i, 1);
      }

      animFrame = requestAnimationFrame(draw);
    }

    draw();

    const onResize = () => {
      W = canvas.width  = window.innerWidth;
      H = canvas.height = window.innerHeight;
      cellW = W / COLS;
      cellH = H / ROWS;
    };
    window.addEventListener('resize', onResize);
    return () => { cancelAnimationFrame(animFrame); window.removeEventListener('resize', onResize); };
  });

  onDestroy(() => { cancelAnimationFrame(animFrame); unsubTheme(); });
</script>

<canvas bind:this={canvas} class="hud-bg"></canvas>

<style>
  .hud-bg {
    position: fixed; inset: 0;
    width: 100vw; height: 100vh;
    pointer-events: none; z-index: 0;
  }
</style>
