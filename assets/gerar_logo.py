"""Gera o logo oficial da RRS - Riva Rust Store (selo laranja com o caranguejo branco e "RRS").

Mesmo desenho do selo do instalador: fundo laranja (215, 75, 40), cantos arredondados, caranguejo da
fonte Segoe UI Emoji em uma cor só (a mesma que a interface usa para o 🦀) e "RRS" em branco.

Saídas (pasta assets/):
  logo.png        512x512, ícone quadrado (caranguejo em cima, RRS embaixo) - janela e README
  logo-wide.png   selo largo igual ao da interface (caranguejo + RRS lado a lado)
  rrs.ico         ícone do .exe e dos atalhos (16 a 256 px; nos tamanhos pequenos só o caranguejo)

Uso: python assets/gerar_logo.py   (precisa de Pillow e do cache do cargo com epaint_default_fonts)
"""

from pathlib import Path
import glob
import os

from PIL import Image, ImageDraw, ImageFont

ORANGE = (215, 75, 40, 255)
WHITE = (255, 255, 255, 255)
CRAB = "\U0001F980"
HERE = Path(__file__).resolve().parent


def find_font(pattern: str) -> str:
    home = os.environ.get("USERPROFILE") or os.path.expanduser("~")
    hits = sorted(glob.glob(os.path.join(home, ".cargo", "registry", "src", "*", "epaint_default_fonts-*", "fonts", pattern)))
    if not hits:
        raise SystemExit(f"fonte {pattern} não encontrada no cache do cargo (rode cargo build antes)")
    return hits[-1]


# O NotoEmoji do egui não tem o 🦀: na interface ele vem do Segoe UI Emoji (reserva configurada em
# gui::font_definitions), desenhado em uma cor só. O Pillow sem embedded_color faz o mesmo.
EMOJI_FONT = os.path.join(os.environ.get("WINDIR", r"C:\Windows"), "Fonts", "seguiemj.ttf")
TEXT_FONT = find_font("Ubuntu-Light.ttf")


def text_box(draw, txt, font):
    l, t, r, b = draw.textbbox((0, 0), txt, font=font)
    return l, t, r - l, b - t


def square_icon(size: int, with_text: bool) -> Image.Image:
    # Desenha em 4x e reduz: bordas suaves em qualquer tamanho
    s = size * 4
    img = Image.new("RGBA", (s, s), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    d.rounded_rectangle((0, 0, s - 1, s - 1), radius=int(s * 0.22), fill=ORANGE)
    if with_text:
        crab_font = ImageFont.truetype(EMOJI_FONT, int(s * 0.46))
        txt_font = ImageFont.truetype(TEXT_FONT, int(s * 0.30))
        cl, ct, cw, ch = text_box(d, CRAB, crab_font)
        tl, tt, tw, th = text_box(d, "RRS", txt_font)
        gap = int(s * 0.04)
        top = (s - (ch + gap + th)) // 2
        d.text(((s - cw) // 2 - cl, top - ct), CRAB, font=crab_font, fill=WHITE)
        # Traço duplo deixa o "RRS" mais legível nos tamanhos de ícone
        for dx in (0, max(1, s // 256)):
            d.text(((s - tw) // 2 - tl + dx, top + ch + gap - tt), "RRS", font=txt_font, fill=WHITE)
    else:
        crab_font = ImageFont.truetype(EMOJI_FONT, int(s * 0.72))
        cl, ct, cw, ch = text_box(d, CRAB, crab_font)
        d.text(((s - cw) // 2 - cl, (s - ch) // 2 - ct), CRAB, font=crab_font, fill=WHITE)
    return img.resize((size, size), Image.LANCZOS)


def wide_badge(height: int = 160) -> Image.Image:
    s = 4
    h = height * s
    crab_font = ImageFont.truetype(EMOJI_FONT, int(h * 0.52))
    txt_font = ImageFont.truetype(TEXT_FONT, int(h * 0.40))
    probe = ImageDraw.Draw(Image.new("RGBA", (1, 1)))
    cl, ct, cw, ch = text_box(probe, CRAB, crab_font)
    tl, tt, tw, th = text_box(probe, "RRS", txt_font)
    pad_x, gap = int(h * 0.26), int(h * 0.10)
    w = pad_x * 2 + cw + gap + tw
    img = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    d.rounded_rectangle((0, 0, w - 1, h - 1), radius=int(h * 0.24), fill=ORANGE)
    d.text((pad_x - cl, (h - ch) // 2 - ct), CRAB, font=crab_font, fill=WHITE)
    d.text((pad_x + cw + gap - tl, (h - th) // 2 - tt), "RRS", font=txt_font, fill=WHITE)
    return img.resize((w // s, height), Image.LANCZOS)


def main():
    square_icon(512, True).save(HERE / "logo.png")
    wide_badge().save(HERE / "logo-wide.png")
    sizes = [16, 24, 32, 48, 64, 128, 256]
    frames = [square_icon(n, n >= 48) for n in sizes]
    frames[-1].save(HERE / "rrs.ico", format="ICO", sizes=[(n, n) for n in sizes], append_images=frames[:-1])
    print("ok:", ", ".join(p.name for p in (HERE / "logo.png", HERE / "logo-wide.png", HERE / "rrs.ico")))


if __name__ == "__main__":
    main()
