# Article editor v2: Markdown writing view

The v2 redesign replaces the block editor as the default editing surface with
one full-document Markdown source and a live rendered preview, closer to
[huasheng_editor](https://github.com/alchaincyf/huasheng_editor).

- **Desktop:** split view, with source on the left and a themed preview on a
  paper-width card on the right. A 编辑 | 双栏 | 预览 switch and a style
  (theme) panel sit in the header.
- **Phone width:** 编辑 | 预览, a title field above the source, and
  formatting at the bottom.

## Design atlases (image-to-appcard-flow)

| Atlas | Scenes | Artboard | Generation |
|---|---|---|---|
| `source/atlas-v2-mobile.png` | 8: article list, Markdown editing, live preview, theme sheet, insert image, cover and summary, choose chat, published | 406×776 | `gpt-image-2`, requested and actual 2048×2048 |
| `source/atlas-v2-desktop.png` | 4: split editor, theme gallery, paste images, publish sheet | 1280×800 | `gpt-image-2`, requested and actual 2560×1600 |

- **Records:** the exact prompts and receipts (model, sizes, SHA-256) are
  alongside the atlases. No credentials are recorded.
- **Mobile intake:** the upstream flow ran `intake` and `prepare`
  (`image-to-appcard-flow-v2-mobile.json`, output in
  `pipeline-output/intake-v2-mobile`).
- **Desktop intake (local extension):** upstream `flow.py` accepts only the
  406×776 artboard, and `atlas.py` requires 8–12 scenes.
  `intake_desktop.py` runs the unmodified upstream `atlas.py` with only the
  scene-count check relaxed. Crops, reversible transforms and hashes are
  upstream's. Output is in `pipeline-output/intake-v2-desktop`.

## Instrument verification

The same code path as the flow's Studio-less instrument route:
- **Instrument mode:** the real Rinx binary runs with hidden windows
  (`MAKEPAD_HIDE_WINDOWS=1`) and is driven through its remote-control port
  (`MAKEPAD_REMOTE`), using the signed-in demo fixture profile.
- **Comparison:** `compare_v2.py` compares window grabs with the intake
  references, using `compare_screens.py`'s side-by-side and diff-region
  functions. Output is in `evidence-v2/`. These are evidence about pixels,
  not a visual acceptance or a similarity score.

| Scene | Result after fixes |
|---|---|
| split-editor | Header (文章编辑器, centred title, style pill, 编辑/双栏/预览, 已保存, 发布), toolbar, source, live preview card, three-image gallery row, stats line |
| theme-gallery | 3×4 swatch grid; 金融时报 re-themes the preview and the pill |
| markdown-editing (phone) | Back, 编辑/预览, 发布; title field; source; 已保存 and bottom toolbar |
| live-preview (phone) | Title, quote, heading, list, three-image row |

Defects found by looking at the side-by-sides and fixed:
- CJK text rendered as boxes in the monospace source font.
- The title was repeated in the preview.
- The source pane background was grey.
- The header overflowed at phone width, hiding 预览 and 发布.
- Three images rendered stacked full-width instead of as one row.
- The theme list needed scrolling.

The toolbar was exercised through the instrument: **B** wraps or inserts
`**…**`, and **H** prefixes the line with `## `.

## Known gaps (not yet matching the atlases, or not verified)

- **Source pane:** no line numbers or Markdown syntax colouring; it is a plain
  text input.
- **Accent colours:** the quote bar and headings use the ink colour rather
  than the theme accent.
- **Favourites:** the theme gallery has no favourite stars.
- **Image references:** these are full `asset:<64-hex>` IDs, not short
  aliases.
- **Dropping image files:** implemented, with an overlay and import at the
  cursor, but **not verified in the running app**: the remote bridge has no
  drag-and-drop route.
- **Images:** they are resized to 2048 px and stored as PNG (the existing
  importer), not JPEG-compressed. The "自动压缩" wording is
  aspirational until then.
- **No native implementation yet for the remaining scenes:** paste-images
  (beyond the overlay), the publish sheet (发布 goes to the existing
  publication review), and the mobile theme sheet, insert-image, cover,
  chat and published scenes. These still use the existing pages.
- **Block editor:** it is no longer reachable from the UI (`block_mode`). The
  older `tools/wechat-ux/live/native_article_*.py` scripts that drive it need
  updating.
