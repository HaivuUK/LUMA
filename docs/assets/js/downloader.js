const OWNER = 'HaivuUK';
const REPO = 'LUMA';
let cachedAssets = null;

const HOVER_COLOR = 'rgb(76 11 134 / 0.65)';

document.addEventListener('DOMContentLoaded', () => {
    const platform = getPlatform();
    const btn = document.getElementById('download-btn');
    const textSpan = document.getElementById('download-text');

    if (btn) {
        const icon = btn.querySelector(`[data-platform="${platform}"]`);
        if (icon) icon.style.display = 'inline-flex';

        textSpan.innerText = `Download`;

        btn.addEventListener('click', async (e) => {
            e.preventDefault();
            await handleDownloadFlow();
        });
    }
});

async function handleDownloadFlow() {
    const platform = getPlatform();
    const btn = document.getElementById('download-btn');
    const textSpan = document.getElementById('download-text');
    const wrapper = document.getElementById('download-menu-wrapper');
    const menu = document.getElementById('download-menu');
    const originalText = textSpan.innerText;

    if (wrapper.classList.contains('hx-opacity-100')) {
        closeMenu(wrapper, btn);
        return;
    }

    if (cachedAssets) {
        showLinuxMenu(platform, cachedAssets, menu, wrapper, btn);
        handleDirectDownload(platform, cachedAssets);
        return;
    }

    try {
        textSpan.innerText = 'Searching...';
        const response = await fetch(`https://api.github.com/repos/${OWNER}/${REPO}/releases/latest`);
        if (!response.ok) throw new Error('API Error');

        const data = await response.json();
        cachedAssets = data.assets;

        showLinuxMenu(platform, cachedAssets, menu, wrapper, btn);
        handleDirectDownload(platform, cachedAssets);

        textSpan.innerText = originalText;
    } catch (error) {
        console.error(error);
        textSpan.innerText = originalText;
        alert('Could not connect to GitHub.');
    }
}

function showLinuxMenu(platform, assets, menu, wrapper, btn) {
    if (platform !== 'linux') return;

    const linuxAssets = assets.filter(a =>
        ['.appimage', '.deb', '.rpm'].some(ext => a.name.toLowerCase().endsWith(ext))
    );

    if (linuxAssets.length > 0) {
        menu.innerHTML = '';
        linuxAssets.forEach((asset, index) => {
            const item = document.createElement('a');
            item.href = asset.browser_download_url;
            item.innerText = asset.name;
            item.title = asset.name; // Shows full name on hover if truncated

            const lowerName = asset.name.toLowerCase();
            if (lowerName.endsWith('.appimage')) {
                item.innerText = 'AppImage';
            } else if (lowerName.endsWith('.deb')) {
                item.innerText = 'Deb';
            } else if (lowerName.endsWith('.rpm')) {
                item.innerText = 'RPM';
            } else {
                item.innerText = asset.name;
            }

            item.className = "hx-block hx-px-4 hx-py-3 hx-text-sm hx-text-black dark:hx-text-white hx-truncate hx-text-center";
            item.style.transition = 'background-color 0.2s';
            item.style.cursor = 'pointer';

            // Add faint borders between items, except the last one
            if (index !== linuxAssets.length - 1) {
                item.style.borderBottom = '1px solid rgba(128, 128, 128, 0.2)';
            }

            // JavaScript hover effect
            item.addEventListener('mouseenter', () => item.style.backgroundColor = HOVER_COLOR);
            item.addEventListener('mouseenter', () => item.style.color = 'rgb(234 231 231 / 0.95)');
            item.addEventListener('mouseleave', () => item.style.backgroundColor = 'transparent');
            item.addEventListener('mouseout', () => item.style.color = '');

            menu.appendChild(item);
        });

        // --- BLENDING LOGIC ---
        const btnStyle = window.getComputedStyle(btn);
        const borderBottom = parseFloat(btnStyle.borderBottomWidth) || 0;
        wrapper.style.marginTop = `-${borderBottom}px`;

        btn.style.borderBottomLeftRadius = '0px';
        btn.style.borderBottomRightRadius = '0px';

        btn.style.borderBottomColor = 'transparent';

        // Animate in
        wrapper.classList.remove('hx-opacity-0', 'hx-pointer-events-none');
        wrapper.classList.add('hx-opacity-100', 'hx-pointer-events-auto', 'hx-translate-y-0');
    }
}

function handleDirectDownload(platform, assets) {
    if (platform === 'linux') return;

    const priorities = platform === 'windows' ? ['.exe', '.msi'] : ['.pkg', '.dmg', '.app'];
    let bestAsset = null;
    for (const ext of priorities) {
        bestAsset = assets.find(a => a.name.toLowerCase().endsWith(ext));
        if (bestAsset) break;
    }
    if (bestAsset) {
        window.location.href = bestAsset.browser_download_url;
    }
}

function closeMenu(wrapper, btn) {
    wrapper.classList.add('hx-opacity-0', 'hx-pointer-events-none');
    wrapper.classList.remove('hx-opacity-100', 'hx-pointer-events-auto');

    // Restore the button's standard appearance after closing
    if (btn) {
        btn.style.borderBottomLeftRadius = '';
        btn.style.borderBottomRightRadius = '';
        btn.style.borderBottomColor = '';
    }
}

function getPlatform() {
    const ua = window.navigator.userAgent.toLowerCase();
    if (ua.includes('win')) return 'windows';
    if (ua.includes('mac')) return 'macos';
    if (ua.includes('linux')) return 'linux';
    return 'unknown';
}

window.addEventListener('click', (e) => {
    const wrapper = document.getElementById('download-menu-wrapper');
    const btn = document.getElementById('download-btn');
    if (wrapper && !btn.contains(e.target) && !wrapper.contains(e.target)) {
        closeMenu(wrapper, btn);
    }
});