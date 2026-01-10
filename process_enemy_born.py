#!/usr/bin/env python3
"""
将敌方坦克出生动画帧缩放到坦克大小并合成精灵图
"""
from PIL import Image
import os

# 配置
SOURCE_DIR = "/home/nanbert/box/enemy_born/frames_new"
OUTPUT_DIR = "/home/nanbert/rust/tank_battle/assets"
TANK_WIDTH = 87
TANK_HEIGHT = 87
SPRITE_SHEET_NAME = "enemy_born_sheet.png"

def process_images():
    # 获取所有图片文件并排序
    files = sorted([f for f in os.listdir(SOURCE_DIR) if f.endswith('.png')])

    if not files:
        print(f"在 {SOURCE_DIR} 中没有找到 PNG 文件")
        return

    print(f"找到 {len(files)} 帧图片")

    # 缩放图片
    resized_images = []
    for i, filename in enumerate(files, 1):
        filepath = os.path.join(SOURCE_DIR, filename)
        img = Image.open(filepath).convert('RGBA')

        # 缩放到坦克大小
        resized = img.resize((TANK_WIDTH, TANK_HEIGHT), Image.Resampling.LANCZOS)
        resized_images.append(resized)
        print(f"处理 {i}/{len(files)}: {filename}")

    # 计算精灵图布局
    num_images = len(resized_images)
    cols = 4  # 每行4帧
    rows = (num_images + cols - 1) // cols

    # 创建精灵图
    sprite_sheet = Image.new('RGBA', (cols * TANK_WIDTH, rows * TANK_HEIGHT))

    # 将所有帧拼接到精灵图上
    for idx, img in enumerate(resized_images):
        x = (idx % cols) * TANK_WIDTH
        y = (idx // cols) * TANK_HEIGHT
        sprite_sheet.paste(img, (x, y))

    # 保存精灵图
    output_path = os.path.join(OUTPUT_DIR, SPRITE_SHEET_NAME)
    sprite_sheet.save(output_path)
    print(f"\n精灵图已保存到: {output_path}")
    print(f"精灵图尺寸: {sprite_sheet.size[0]} x {sprite_sheet.size[1]}")
    print(f"布局: {rows} 行 x {cols} 列")

    # 输出精灵图布局信息
    print(f"\n精灵图布局信息 (用于 Bevy TextureAtlas):")
    print(f"  纹理大小: {sprite_sheet.size[0]} x {sprite_sheet.size[1]}")
    print(f"  单帧大小: {TANK_WIDTH} x {TANK_HEIGHT}")
    print(f"  总帧数: {num_images}")
    print(f"  列数: {cols}")
    print(f"  行数: {rows}")

if __name__ == "__main__":
    process_images()