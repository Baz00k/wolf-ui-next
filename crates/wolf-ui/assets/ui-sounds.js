(() => {
    if (window.__wolfUiSoundsInstalled) return;
    window.__wolfUiSoundsInstalled = true;

    const SOUND_GAINS = {
        navigate: 0.15,
        select: 0.18,
        back: 0.13,
    };
    const DEFAULT_GAIN = 0.5;

    function createUiSounds(urls = {}) {
        const AudioContext = window.AudioContext || window.webkitAudioContext;
        if (!AudioContext) return { play() {} };

        const context = new AudioContext();
        const sounds = new Map(
            Object.entries(urls)
                .filter(([, url]) => Boolean(url))
                .map(([name, url]) => [name, loadAudioBuffer(context, url)]),
        );

        resumeOnFirstGesture(context);

        return {
            play(name) {
                const buffer = sounds.get(name)?.buffer;
                if (!buffer) return;

                if (context.state === "suspended") {
                    context.resume().catch(() => {});
                }

                const source = context.createBufferSource();
                const gain = context.createGain();
                gain.gain.value = SOUND_GAINS[name] ?? DEFAULT_GAIN;
                source.buffer = buffer;
                source.connect(gain).connect(context.destination);
                source.start();
            },
        };
    }

    function resumeOnFirstGesture(context) {
        const resume = () => {
            if (context.state === "suspended") context.resume().catch(() => {});
        };

        document.addEventListener("pointerdown", resume, { once: true, passive: true });
        document.addEventListener("keydown", resume, { once: true });
    }

    function loadAudioBuffer(context, url) {
        const sound = { buffer: null };
        fetch(url)
            .then((response) => response.arrayBuffer())
            .then((data) => context.decodeAudioData(data))
            .then((buffer) => {
                sound.buffer = buffer;
            })
            .catch((error) => {
                console.warn(`Failed to load UI sound ${url}`, error);
            });

        return sound;
    }

    window.__wolfUiSounds = createUiSounds(window.__wolfUiSoundUrls);
})();
