export const PIECE_RES = new Map<string, HTMLImageElement>();
const PIECE_CODES = ['wP', 'wN', 'wB', 'wR', 'wQ', 'wK', 'bP', 'bN', 'bB', 'bR', 'bQ', 'bK'];

function loadImage(code: string): Promise<HTMLImageElement> {
    return new Promise((resolve, reject) => {
        const img = new Image();
        const url = `https://lichess1.org/assets/piece/cburnett/${code}.svg`;
        img.src = url;
        img.onload = () => resolve(img);
        img.onerror = () => reject(new Error(`Failed to load: ${url}`));
    });
}

export function initializeChess(callback: () => void) {
    Promise.all(PIECE_CODES.map(loadImage))
        .then(images => {
            console.log("✅ All assets loaded");
            images.forEach((img, index) => {
                const code = PIECE_CODES[index];
                const color = code[0];
                const piece = color === 'w' ? code[1] : code[1].toLowerCase();
                PIECE_RES.set(piece, img);
            });
            callback();
        })
        .catch(err => {
            console.error("❌ One or more images failed to load:", err);
        });
}
