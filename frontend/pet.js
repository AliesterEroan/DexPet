const { invoke } = window.__TAURI__.core;
const { getCurrentWindow } = window.__TAURI__.window;
const { PhysicalPosition } = window.__TAURI__.dpi;
const { listen } = window.__TAURI__.event;
const win = getCurrentWindow();
const dex = document.getElementById('dex');
const reduceMotion = matchMedia('(prefers-reduced-motion: reduce)');

dex.addEventListener('mousedown', (e) => {
    if (e.button === 0) win.startDragging();
});

dex.addEventListener('click', () => {
    dex.classList.remove('boing');
    void dex.offsetWidth;
    dex.classList.add('boing');
});

dex.addEventListener('keydown', (e) => {
    const step = e.shiftKey ? 1 : 24;
    const moves = {
        ArrowLeft: [-step, 0], ArrowRight: [step, 0],
        ArrowUp: [0, -step], ArrowDown: [0, step],
    };
    const move = moves[e.key];
    if (!move) return;
    e.preventDefault();
    win.outerPosition().then((p) => {
        win.setPosition(new PhysicalPosition(p.x + move[0], p.y + move[1]));
    });
});

function reportHitbox() {
    const r = dex.getBoundingClientRect();
    const box = { x: r.left, y: r.top, w: r.width, h: r.height };
    window.__hitbox = box;
    invoke('set_pet_hitbox', box)
        .then(() => { window.__hitboxOk = true; })
        .catch((e) => { window.__hitboxErr = String(e); });
}
reportHitbox();
addEventListener('resize', reportHitbox);

let saveTimer = null;
win.onMoved(({ payload }) => {
    clearTimeout(saveTimer);
    saveTimer = setTimeout(() => {
        invoke('save_pet_position', { x: payload.x, y: payload.y });
    }, 400);
});

listen('pet-cursor', ({ payload: [x, y, w, h] }) => {
    if (!awake) return;
    const dx = Math.max(-1, Math.min(1, (x - w / 2) / (w / 2)));
    const dy = Math.max(-1, Math.min(1, (y - h / 2) / (h / 2)));
    dex.style.setProperty('--look-x', dx.toFixed(3));
    dex.style.setProperty('--look-y', dy.toFixed(3));
});

// Idle life: a sparse blink on the promoted eye layer. Rust's pet-visible event is
// the source of truth for hide/show; while asleep nothing animates or is scheduled.
let awake = true;
let blinkTimer = null;
function scheduleBlink() {
    clearTimeout(blinkTimer);
    if (!awake || reduceMotion.matches) return;
    blinkTimer = setTimeout(() => {
        dex.classList.add('blink');
        setTimeout(() => dex.classList.remove('blink'), 90);
        scheduleBlink();
    }, 5000 + Math.random() * 4000);
}
function setAwake(on) {
    awake = on;
    dex.classList.toggle('asleep', !on);
    if (on) scheduleBlink();
    else clearTimeout(blinkTimer);
}
listen('pet-visible', ({ payload }) => setAwake(!!payload));
document.addEventListener('visibilitychange', () => {
    if (document.hidden) setAwake(false);
});
reduceMotion.addEventListener('change', () => setAwake(awake));
setAwake(true);