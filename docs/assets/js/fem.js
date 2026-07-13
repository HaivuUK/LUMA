console.log("fem.js loaded with multi-instance support");

(function () {
  // ---------------- Viridis Color Scale ----------------
  const viridis = [
    [68, 1, 84],
    [59, 82, 139],
    [33, 145, 140],
    [94, 201, 98],
    [253, 231, 37]
  ];

  const lerp = (a, b, t) => a + (b - a) * t;

  function color(t) {
    t = Math.max(0, Math.min(1, t));
    const n = viridis.length - 1;
    const i = Math.floor(t * n);
    const f = t * n - i;

    const a = viridis[i];
    const b = viridis[Math.min(i + 1, n)];

    return `rgb(
      ${Math.round(lerp(a[0], b[0], f))},
      ${Math.round(lerp(a[1], b[1], f))},
      ${Math.round(lerp(a[2], b[2], f))}
    )`;
  }

  // ---------------- MESH INSTANCE INITIALIZER ----------------
  function initMeshInstance(containerSelector) {
    const container = document.querySelector(containerSelector);
    if (!container) return false; // Exit quietly if the section isn't on the current page

    const svg = container.querySelector("svg.fem");
    if (!svg) return false;

    // Read custom row counts via data attribute, falling back to 12 if undefined
    const rowCount = parseInt(container.getAttribute("data-rows"), 10) || 12;

    let W = 0;
    let H = 0;
    let polys = [];
    let tris = [];

    // Setup localized fluid hotspots for this specific background instance
    const hotspots = Array.from({ length: 4 }, () => ({
      x: Math.random(),
      y: Math.random(),
      vx: (Math.random() - 0.5) * 0.0015,
      vy: (Math.random() - 0.5) * 0.0015,
      r: 0.25 + Math.random() * 0.25
    }));

    function buildMesh() {
      W = container.clientWidth;
      H = container.clientHeight;

      svg.setAttribute("viewBox", `0 0 ${W} ${H}`);
      svg.innerHTML = "";

      const dy = H / rowCount;
      const dx = dy * 1.4; // Maintains clean triangle aspect ratio
      const cols = Math.ceil(W / dx);
      const actualDx = W / cols;

      const grid = [];
      for (let y = 0; y <= rowCount; y++) {
        for (let x = 0; x <= cols; x++) {
          grid.push({ x: x * actualDx, y: y * dy });
        }
      }

      tris = [];
      for (let y = 0; y < rowCount; y++) {
        for (let x = 0; x < cols; x++) {
          const i = y * (cols + 1) + x;
          const a = grid[i];
          const b = grid[i + 1];
          const c = grid[i + cols + 1];
          const d = grid[i + cols + 2];

          if ((x + y) % 2 === 0) {
            tris.push([a, b, d]);
            tris.push([a, d, c]);
          } else {
            tris.push([a, b, c]);
            tris.push([b, d, c]);
          }
        }
      }

      polys = tris.map(t => {
        const p = document.createElementNS("http://www.w3.org/2000/svg", "polygon");
        p.setAttribute("stroke-width", "0.5");
        svg.appendChild(p);
        return { t, p };
      });
    }

    function field(x, y, t) {
      let v = 0;
      for (const h of hotspots) {
        const dx = x - h.x;
        const dy = y - h.y;
        v += Math.exp(-(dx * dx + dy * dy) / (h.r * h.r));
      }
      v += 0.25 * Math.sin(x * 1.5 + t);
      v += 0.25 * Math.cos(y * 1.5 - t * 0.8);
      return v;
    }

    function animate() {
      const time = performance.now() * 0.0005; // Calming speed

      W = container.clientWidth;
      H = container.clientHeight;

      for (const h of hotspots) {
        h.x += h.vx;
        h.y += h.vy;
        if (h.x < 0 || h.x > 1) h.vx *= -1;
        if (h.y < 0 || h.y > 1) h.vy *= -1;
      }

      for (const o of polys) {
        const t = o.t;
        let v = (field(t[0].x / W, t[0].y / H, time) +
                 field(t[1].x / W, t[1].y / H, time) +
                 field(t[2].x / W, t[2].y / H, time)) / 3;

        v = (v - 0.1) / 1.6; // Scale down spectrum spikes safely

        const fillColor = color(v);
        o.p.setAttribute("fill", fillColor);
        o.p.setAttribute("stroke", fillColor);

        o.p.setAttribute("points", `
          ${t[0].x},${t[0].y} 
          ${t[1].x},${t[1].y} 
          ${t[2].x},${t[2].y}
        `);
      }

      requestAnimationFrame(animate);
    }

    // Re-render triangles on browser resize
    window.addEventListener("resize", () => {
      buildMesh();
    });

    buildMesh();
    animate();
    return true;
  }

  // ---------------- ENTRY POINT WAITER ----------------
  // Periodically checks the DOM until elements arrive via markdown/HTML injectors
  function bootstrap() {
    const hasHero = document.querySelector(".hero-fem");
    const hasFooter = document.querySelector(".footer-fem");

    // Keep checking if components are missing but we expect them
    if (!hasHero && !hasFooter) {
      setTimeout(bootstrap, 50);
      return;
    }

    // Safely fire up instances independently based on layout visibility
    initMeshInstance(".hero-fem");
    initMeshInstance(".footer-fem");
  }

  bootstrap();
})();