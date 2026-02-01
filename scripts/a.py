import matplotlib.pyplot as plt
import matplotlib.patches as patches
import japanize_matplotlib

# --- 共通のヘルパー関数 ---

def draw_slide_frame(ax, title):
    """スライドの共通フレーム（ヘッダー、フッターデザイン）を描画する"""
    # ヘッダー（青色の帯）
    header = patches.Rectangle((0, 8), 16, 1, facecolor='#3d5afe', edgecolor='none')
    ax.add_patch(header)
    # タイトル
    ax.text(0.5, 8.5, title, fontsize=24, color='white', fontweight='bold', va='center')

    # フッターのデザイン（右下の斜線）
    for i in range(3):
        line = patches.Polygon(
            [[14.5 + i * 0.4, 0], [15.0 + i * 0.4, 0], [15.5 + i * 0.4, 0.8], [15.0 + i * 0.4, 0.8]],
            closed=True, facecolor='#fbc02d', edgecolor='none'
        )
        ax.add_patch(line)
        
    # フッターの青い帯
    footer = patches.Rectangle((0, 0), 16, 0.8, facecolor='#3d5afe', edgecolor='none', zorder=0)
    ax.add_patch(footer)


def draw_grid(ax, x, y, width, height, rows, cols, label):
    """格子状の配列図を描画する"""
    cell_w = width / cols
    cell_h = height / rows
    for r in range(rows):
        for c in range(cols):
            rect = patches.Rectangle(
                (x + c * cell_w, y + (rows - 1 - r) * cell_h),
                cell_w, cell_h, fill=False, edgecolor='black', lw=1
            )
            ax.add_patch(rect)
    # ラベル
    ax.text(x + width / 2, y + height + 0.3, label, ha='center', fontsize=14)

# --- 1枚目：実装の仕組み ---

def create_slide_1_implementation():
    fig, ax = plt.figure(figsize=(16, 9)), plt.gca()
    ax.set_xlim(0, 16); ax.set_ylim(0, 9); ax.axis('off')
    draw_slide_frame(ax, "実装戦略：領域分割と共有メモリによる並列化")

    # 左側の図（現時刻のデータ配列）
    grid_x, grid_y = 1, 2.5
    grid_w, grid_h = 6, 4.5
    rows, cols = 8, 8
    draw_grid(ax, grid_x, grid_y, grid_w, grid_h, rows, cols, "現時刻のデータ配列")

    # 領域分割の矢印とラベル
    # スレッド0 (上半分)
    ax.annotate('', xy=(grid_x - 0.5, grid_y + grid_h), xytext=(grid_x - 0.5, grid_y + grid_h / 2),
                arrowprops=dict(arrowstyle='<->', lw=2))
    ax.text(grid_x - 0.8, grid_y + grid_h * 3 / 4, "スレッド0\n担当領域", ha='right', va='center', fontsize=16)
    
    # スレッド1 (下半分)
    ax.annotate('', xy=(grid_x - 0.5, grid_y + grid_h / 2), xytext=(grid_x - 0.5, grid_y),
                arrowprops=dict(arrowstyle='<->', lw=2))
    ax.text(grid_x - 0.8, grid_y + grid_h / 4, "スレッド1\n担当領域", ha='right', va='center', fontsize=16)

    # 右側の説明テキスト
    text_x = 8.5
    text_y = 6.5
    ax.text(text_x, text_y, "■ 実装のアプローチ", fontsize=20, fontweight='bold', color='#3d5afe')
    
    bullets = [
        "ダブルバッファリング方式を採用",
        "  - 「現時刻」と「次の時刻」の2つの配列を用意",
        "",
        "領域分割によるマルチスレッド化",
        "  - 複数のスレッドでメモリ領域を共有",
        "  - 担当エリアを分割（例：上下）",
        "  - 各スレッドがメモリへ直接書き込み（高速化）"
    ]
    for i, line in enumerate(bullets):
        ax.text(text_x, text_y - 1.0 - i * 0.6, line, fontsize=16, linespacing=1.5)

    plt.savefig("slide_1_implementation.png", dpi=100, bbox_inches='tight')
    plt.close()

# --- 2枚目：データ競合のリスク ---

def create_slide_2_risk():
    fig, ax = plt.figure(figsize=(16, 9)), plt.gca()
    ax.set_xlim(0, 16); ax.set_ylim(0, 9); ax.axis('off')
    draw_slide_frame(ax, "共有メモリにおけるリスク：データ競合の危険性")

    # 右側の図（次の時刻のデータ配列）
    grid_x, grid_y = 8.5, 2.5
    grid_w, grid_h = 6, 4.5
    rows, cols = 8, 8
    draw_grid(ax, grid_x, grid_y, grid_w, grid_h, rows, cols, "次の時刻のデータ配列")

    # アクセス矢印
    # thread0
    ax.annotate('', xy=(grid_x + 2, grid_y + grid_h / 2 + 0.5), xytext=(grid_x - 1.5, grid_y + grid_h / 2 + 0.5),
                arrowprops=dict(arrowstyle='->', lw=2))
    ax.text(grid_x - 1.7, grid_y + grid_h / 2 + 0.5, "thread0\n直接アクセス", ha='right', va='center', fontsize=12)

    # thread1
    ax.annotate('', xy=(grid_x + 2, grid_y + grid_h / 2 - 0.5), xytext=(grid_x - 1.5, grid_y + grid_h / 2 - 0.5),
                arrowprops=dict(arrowstyle='->', lw=2))
    ax.text(grid_x - 1.7, grid_y + grid_h / 2 - 0.5, "thread1\n直接アクセス", ha='right', va='center', fontsize=12)

    # 警告（データ競合）
    warning_x = grid_x + grid_w - 1
    warning_y = grid_y + grid_h + 0.8
    ax.text(warning_x, warning_y, "▲ データ競合の可能性", fontsize=16, color='red', fontweight='bold', ha='right')
    # 境界付近のセルを赤枠で強調
    boundary_y = grid_y + grid_h / 2
    cell_h = grid_h / rows
    rect = patches.Rectangle((grid_x, boundary_y - cell_h), grid_w, cell_h*2, fill=False, edgecolor='red', lw=3, ls='--')
    ax.add_patch(rect)


    # 左側の説明テキスト
    text_x = 1
    text_y = 6.5
    ax.text(text_x, text_y, "■ 潜んでいる危険性", fontsize=20, fontweight='bold', color='#d32f2f')

    bullets = [
        "境界部分のアクセス管理が困難",
        "  - 隣接データへのアクセスでスレッドが干渉",
        "",
        "プログラマへの重い責任",
        "  - アクセス管理を「完璧」に行う必要がある",
        "  - わずかなミスがバグにつながる",
        "",
        "データ競合（Data Race）の発生",
        "  - 計算結果の破損やクラッシュの原因"
    ]
    for i, line in enumerate(bullets):
        ax.text(text_x, text_y - 1.0 - i * 0.6, line, fontsize=16, linespacing=1.5)

    plt.savefig("slide_2_risk.png", dpi=100, bbox_inches='tight')
    plt.close()

# --- 実行 ---
if __name__ == '__main__':
    create_slide_1_implementation()
    print("slide_1_implementation.png を生成しました。")
    create_slide_2_risk()
    print("slide_2_risk.png を生成しました。")