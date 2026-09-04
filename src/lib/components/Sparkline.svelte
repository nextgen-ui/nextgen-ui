<script lang="ts">
  import { onMount, onDestroy } from 'svelte';

  export let data: number[] = [];
  export let max: number = 100;
  export let color: string = '#00c8ff';
  export let height: number = 32;
  export let width: number = 160;
  export let filled: boolean = true;

  let canvas: HTMLCanvasElement;

  function render() {
    if (!canvas || (typeof document !== 'undefined' && document.hidden)) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const dpr = typeof window !== 'undefined' ? window.devicePixelRatio || 1 : 1;
    const physW = Math.round(width * dpr);
    const physH = Math.round(height * dpr);

    if (canvas.width !== physW || canvas.height !== physH) {
      canvas.width = physW;
      canvas.height = physH;
    }

    ctx.resetTransform();
    ctx.scale(dpr, dpr);
    ctx.clearRect(0, 0, width, height);

    const len = data.length;
    if (len === 0) return;

    const safeMax = max > 0 ? max : 1;
    const padX = 2;
    const padY = 2;
    const effH = height - padY * 2;

    if (len === 1) {
      const y = padY + effH - (Math.min(data[0], safeMax) / safeMax) * effH;
      ctx.beginPath();
      ctx.arc(width - padX, y, 2, 0, Math.PI * 2);
      ctx.fillStyle = color;
      ctx.fill();
      return;
    }

    const drawW = width - padX * 2;
    const step = drawW / (len - 1);

    // Line stroke path
    ctx.beginPath();
    for (let i = 0; i < len; i++) {
      const x = padX + i * step;
      const y = padY + effH - (Math.min(data[i], safeMax) / safeMax) * effH;
      if (i === 0) ctx.moveTo(x, y);
      else ctx.lineTo(x, y);
    }

    ctx.strokeStyle = color;
    ctx.lineWidth = 1.5;
    ctx.lineCap = 'round';
    ctx.lineJoin = 'round';
    ctx.globalAlpha = 1.0;
    ctx.stroke();

    // Fill area below line
    if (filled) {
      ctx.lineTo(width - padX, height);
      ctx.lineTo(padX, height);
      ctx.closePath();
      ctx.fillStyle = color;
      ctx.globalAlpha = 0.12;
      ctx.fill();
    }

    // Current value dot at latest point
    const lastVal = data[len - 1];
    const lastY = padY + effH - (Math.min(lastVal, safeMax) / safeMax) * effH;
    ctx.beginPath();
    ctx.arc(width - padX, lastY, 2, 0, Math.PI * 2);
    ctx.fillStyle = color;
    ctx.globalAlpha = 1.0;
    ctx.fill();
  }

  $: {
    data; max; color; height; width; filled;
    render();
  }

  function onVisibilityChange() {
    if (!document.hidden) render();
  }

  onMount(() => {
    render();
    document.addEventListener('visibilitychange', onVisibilityChange);
  });

  onDestroy(() => {
    if (typeof document !== 'undefined') {
      document.removeEventListener('visibilitychange', onVisibilityChange);
    }
  });
</script>

<canvas
  bind:this={canvas}
  class="sparkline"
  style="width:{width}px;height:{height}px;"
></canvas>

<style>
  .sparkline {
    display: block;
  }
</style>
