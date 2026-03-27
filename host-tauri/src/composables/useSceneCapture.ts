import type { RenderState } from "../types/render-state";
import { createLogger } from "./useLogger";

const log = createLogger("scene-capture");

const THUMB_W = 384;
const THUMB_H = 216;
const DESIGN_WIDTH = 1920;

function loadImage(src: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.crossOrigin = "anonymous";
    img.onload = () => resolve(img);
    img.onerror = reject;
    img.src = src;
  });
}

/** Draw image in "cover" mode: fill canvas, crop excess */
function drawCover(ctx: CanvasRenderingContext2D, img: HTMLImageElement, cw: number, ch: number) {
  const imgRatio = img.naturalWidth / img.naturalHeight;
  const canvasRatio = cw / ch;
  let sw: number, sh: number, sx: number, sy: number;

  if (imgRatio > canvasRatio) {
    sh = img.naturalHeight;
    sw = sh * canvasRatio;
    sx = (img.naturalWidth - sw) / 2;
    sy = 0;
  } else {
    sw = img.naturalWidth;
    sh = sw / canvasRatio;
    sx = 0;
    sy = (img.naturalHeight - sh) / 2;
  }

  ctx.drawImage(img, sx, sy, sw, sh, 0, 0, cw, ch);
}

/**
 * Capture the current game scene (background + characters) as a base64 PNG.
 * Returns the raw base64 string (without data URI prefix), or null on failure.
 */
export async function captureScene(
  renderState: Readonly<RenderState>,
  assetUrl: (path: string | null | undefined) => string | undefined,
): Promise<string | null> {
  const canvas = document.createElement("canvas");
  canvas.width = THUMB_W;
  canvas.height = THUMB_H;
  const ctx = canvas.getContext("2d");
  if (!ctx) return null;

  const bgUrl = assetUrl(renderState.current_background);
  if (bgUrl) {
    try {
      const bgImg = await loadImage(bgUrl);
      drawCover(ctx, bgImg, THUMB_W, THUMB_H);
    } catch {
      log.warn("failed to load background for thumbnail");
      ctx.fillStyle = "black";
      ctx.fillRect(0, 0, THUMB_W, THUMB_H);
    }
  } else {
    ctx.fillStyle = "black";
    ctx.fillRect(0, 0, THUMB_W, THUMB_H);
  }

  const chars = Object.values(renderState.visible_characters)
    .filter((c) => c.target_alpha > 0 && !c.fading_out)
    .sort((a, b) => a.z_order - b.z_order);

  const thumbScale = THUMB_W / DESIGN_WIDTH;

  for (const char of chars) {
    const charUrl = assetUrl(char.texture_path);
    if (!charUrl) continue;

    try {
      const charImg = await loadImage(charUrl);

      const sx = thumbScale * char.render_scale * char.scale_x;
      const sy = thumbScale * char.render_scale * char.scale_y;
      const drawW = charImg.naturalWidth * Math.abs(sx);
      const drawH = charImg.naturalHeight * Math.abs(sy);

      const anchorPxX = char.anchor_x * drawW;
      const anchorPxY = char.anchor_y * drawH;

      const drawX = char.pos_x * THUMB_W - anchorPxX + char.offset_x * thumbScale;
      const drawY = char.pos_y * THUMB_H - anchorPxY + char.offset_y * thumbScale;

      ctx.globalAlpha = char.target_alpha;

      if (sx < 0) {
        ctx.save();
        ctx.translate(drawX + drawW, drawY);
        ctx.scale(-1, 1);
        ctx.drawImage(charImg, 0, 0, drawW, drawH);
        ctx.restore();
      } else {
        ctx.drawImage(charImg, drawX, drawY, drawW, drawH);
      }
    } catch {
      log.warn(`failed to load character for thumbnail: ${char.texture_path}`);
    }
  }

  ctx.globalAlpha = 1.0;

  try {
    const dataUrl = canvas.toDataURL("image/png");
    const prefix = "data:image/png;base64,";
    return dataUrl.startsWith(prefix) ? dataUrl.slice(prefix.length) : dataUrl;
  } catch {
    log.error("canvas tainted, cannot export thumbnail");
    return null;
  }
}
