//! UI 本地化文本常量
//!
//! 统一管理所有 UI 相关的本地化文本常量

use crate::constants::{LocalizedText, LocalizedTextFormat};

// ==================== 菜单界面文本 ====================

/// 主菜单标题
pub const MENU_TITLE: LocalizedText = LocalizedText {
    cn: "钢铁指令",
    en: "Steel Command",
};

/// 菜单选项
pub const MENU_OPTION_1P: LocalizedText = LocalizedText {
    cn: "单人游戏",
    en: "1 Player",
};

pub const MENU_OPTION_2P: LocalizedText = LocalizedText {
    cn: "双人对战",
    en: "2 Player",
};

pub const MENU_OPTION_LANGUAGE: LocalizedText = LocalizedText {
    cn: "语言 / Language",
    en: "语言 / Language",
};

pub const MENU_OPTION_ABOUT: LocalizedText = LocalizedText {
    cn: "关于",
    en: "About",
};

pub const MENU_OPTION_CREDITS: LocalizedText = LocalizedText {
    cn: "制作人员",
    en: "Credits",
};

pub const MENU_OPTION_EXIT: LocalizedText = LocalizedText {
    cn: "退出",
    en: "EXIT",
};

/// 操作说明文本
pub const CONTROLS_P1: LocalizedText = LocalizedText {
    cn: "玩家1 (李云龙): WASD 移动 | J 射击 | I 召回 | K 冲刺 | L 激光",
    en: "Player 1 (Li Yun Long): WASD to move | J to shoot | I to recall | K to dash | L to laser",
};

pub const CONTROLS_P2: LocalizedText = LocalizedText {
    cn: "玩家2 (楚云飞): 方向键 移动 | 1 射击 | 4 召回 | 2 冲刺 | 3 激光",
    en: "Player 2 (Chu Yun Fei): Arrow Keys to move | 1 to shoot | 4 to recall | 2 to dash | 3 to laser",
};

pub const CONTROLS_GENERAL: LocalizedText = LocalizedText {
    cn: "W/S 选择 | SPACE 确认/暂停 | ESC 退出",
    en: "W/S to select | SPACE to select/pause | ESC to exit",
};

/// 菜单选项文本数组
pub const MENU_OPTIONS: &[LocalizedText; 6] = &[
    MENU_OPTION_1P,
    MENU_OPTION_2P,
    MENU_OPTION_LANGUAGE,
    MENU_OPTION_ABOUT,
    MENU_OPTION_CREDITS,
    MENU_OPTION_EXIT,
];

// ==================== 关于界面文本 ====================

pub const ABOUT_TITLE: LocalizedText = LocalizedText {
    cn: "关于",
    en: "ABOUT",
};

pub const ABOUT_TEXT: LocalizedText = LocalizedText {
    cn: "开发者: 南敬文\n\n        邮箱: 2726905171@qq.com\n\n        版权所有 (c) 2026 南敬文\n        保留所有权利\n\n        本游戏是受《坦克大战 1990》启发的坦克对战游戏.\n        使用 Rust 和 Bevy 游戏引擎开发.\n\n        特别感谢 iFlow 提供的宝贵帮助.\n\n        许可证: MIT 许可证",
    en: "Developer: Nanbert\n\n        Email: 2726905171@qq.com\n\n        Copyright © 2026 Nanbert\n        All rights reserved.\n\n        This is a tank battle game inspired by Battle City 1990.\n        Built with Rust and Bevy game engine.\n\n        Special thanks to iFlow for invaluable assistance.\n\n        License: MIT License",
};

pub const ABOUT_SUPPORT: LocalizedText = LocalizedText {
    cn: "如果你喜欢这个游戏,\n请给我买杯咖啡! (咖啡是程序员的燃料)",
    en: "If you enjoyed the game,\nplease buy me a coffee! ☕️\n(Caffeine is a programmer's fuel)",
};

pub const ABOUT_RETURN: LocalizedText = LocalizedText {
    cn: "按 SPACE 返回",
    en: "Press SPACE to return",
};

pub const PAYMENT_METHOD_ALIPAY: LocalizedText = LocalizedText {
    cn: "支付宝",
    en: "Alipay",
};

pub const PAYMENT_METHOD_WECHAT: LocalizedText = LocalizedText {
    cn: "微信",
    en: "WeChat",
};

// ==================== 致谢界面文本 ====================

pub const CREDITS_TITLE: LocalizedText = LocalizedText {
    cn: "制作人员",
    en: "CREDITS",
};

pub const CREDITS_RETURN: LocalizedText = LocalizedText {
    cn: "按 SPACE 返回",
    en: "Press SPACE to return",
};

pub const CREDITS_TEXT: LocalizedText = LocalizedText {
    cn: "素材来源致谢\n\n\n        OpenGameArt.org:\n        • Bubbles by HorrorPen (CC-BY 3.0)\n        • Explosion by Sinestesia (CC0 1.0)\n        • Laser by netcake3 (CC-BY-SA 3.0/4.0)\n        • Enemy Born by Skorpio (CC-BY 3.0)\n        • Fire Effect by JoesAlotofthings (CC-BY 4.0)\n        • Player/Enemy Tanks & Barrels by irmirx (CC-BY 3.0)\n        • Smoke by Skorpio (CC-BY 3.0)\n        • Hit Spark by Sinestesia (CC0 1.0)\n        • Bullets by Wenrexa (CC0 1.0)\n        • Penetrate Effect by 13rice (CC0 1.0)\n\n\n        通义千问 (AI Generated):\n        • Background, Music Notes (CC0 1.0)\n        • Maps (Brick, Steel, Sea, Tree, Barrier) (CC0 1.0)\n        • Power-ups (10 types) (CC0 1.0)\n        • Track Train (CC0 1.0)\n        • Avatars & Commander (CC0 1.0)\n\n\n        字体:\n        • ChelaOne by Latinotype\n        • Corben\n        • Matemasie\n        • LiuHuanKaTongShouShu by 刘欢\n\n\n        详见 COPYRIGHT 文件。",
    en: "Asset Credits\n\n\n        OpenGameArt.org:\n        • Bubbles by HorrorPen (CC-BY 3.0)\n        • Explosion by Sinestesia (CC0 1.0)\n        • Laser by netcake3 (CC-BY-SA 3.0/4.0)\n        • Enemy Born by Skorpio (CC-BY 3.0)\n        • Fire Effect by JoesAlotofthings (CC-BY 4.0)\n        • Player/Enemy Tanks & Barrels by irmirx (CC-BY 3.0)\n        • Smoke by Skorpio (CC-BY 3.0)\n        • Hit Spark by Sinestesia (CC0 1.0)\n        • Bullets by Wenrexa (CC0 1.0)\n        • Penetrate Effect by 13rice (CC0 1.0)\n\n\n        Tongyi Qianwen (AI Generated):\n        • Background, Music Notes (CC0 1.0)\n        • Maps (Brick, Steel, Sea, Tree, Barrier) (CC0 1.0)\n        • Power-ups (10 types) (CC0 1.0)\n        • Track Train (CC0 1.0)\n        • Avatars & Commander (CC0 1.0)\n\n\n        Fonts:\n        • ChelaOne by Latinotype\n        • Corben\n        • Matemasie\n        • LiuHuanKaTongShouShu by 刘欢\n\n\n        See COPYRIGHT file for full details.",
};

// ==================== 暂停界面文本 ====================

pub const PAUSED_TITLE: LocalizedText = LocalizedText {
    cn: "已暂停",
    en: "PAUSED",
};

pub const PAUSED_INSTRUCTION: LocalizedText = LocalizedText {
    cn: "按 SPACE 继续 | B 返回菜单 | ESC 退出",
    en: "Press SPACE to resume | B to menu | ESC to exit",
};

// ==================== 游戏结束界面文本 ====================

pub const GAME_OVER_TITLE: LocalizedText = LocalizedText {
    cn: "游戏结束",
    en: "GAME OVER",
};

pub const GAME_OVER_RESTART: LocalizedText = LocalizedText {
    cn: "重新开始",
    en: "RESTART",
};

pub const GAME_OVER_MENU: LocalizedText = LocalizedText {
    cn: "返回菜单",
    en: "MENU",
};

pub const GAME_OVER_EXIT: LocalizedText = LocalizedText {
    cn: "退出",
    en: "EXIT",
};

pub const GAME_OVER_INSTRUCTION: LocalizedText = LocalizedText {
    cn: "W/S 选择 | SPACE 确认",
    en: "W/S select | SPACE confirm",
};

// ==================== 能量不足提示文本 ====================

pub const INSUFFICIENT_ENERGY_CN: &str = "能量不足！";
pub const INSUFFICIENT_ENERGY_EN: &str = "Insufficient Energy!";

// ==================== HUD 文本标签 ====================

/// HUD 通用文本标签
pub const HUD_PLAYER1_NAME: LocalizedText = LocalizedText {
    cn: "李云龙",
    en: "Li Yun Long",
};

pub const HUD_PLAYER2_NAME: LocalizedText = LocalizedText {
    cn: "楚云飞",
    en: "Chu Yun Fei",
};

pub const HUD_EFFECTS_TITLE: LocalizedText = LocalizedText {
    cn: "效果",
    en: "Effects",
};

pub const HUD_COMMANDER_LIFE: LocalizedText = LocalizedText {
    cn: "司令官生命:",
    en: "Commander Life:",
};

pub const HUD_ON: LocalizedText = LocalizedText {
    cn: "开启",
    en: "On",
};

pub const HUD_OFF: LocalizedText = LocalizedText {
    cn: "关闭",
    en: "Off",
};

/// HUD 属性标签
pub const HUD_LABEL_SPEED: LocalizedText = LocalizedText {
    cn: "速度:",
    en: "Speed:",
};

pub const HUD_LABEL_FIRE_SPEED: LocalizedText = LocalizedText {
    cn: "射速:",
    en: "Fire Speed:",
};

pub const HUD_LABEL_PROTECTION: LocalizedText = LocalizedText {
    cn: "护盾:",
    en: "Protection:",
};

pub const HUD_LABEL_SHELLS: LocalizedText = LocalizedText {
    cn: "炮弹:",
    en: "Shells:",
};

pub const HUD_LABEL_FIRE_SHELL: LocalizedText = LocalizedText {
    cn: "火焰炮弹",
    en: "Fire Shell",
};

pub const HUD_LABEL_PENETRATE: LocalizedText = LocalizedText {
    cn: "穿透",
    en: "Penetrate",
};

pub const HUD_LABEL_TRACK_CHAIN: LocalizedText = LocalizedText {
    cn: "履带链",
    en: "Track Chain",
};

pub const HUD_LABEL_AIR_CUSHION: LocalizedText = LocalizedText {
    cn: "气垫",
    en: "Air Cushion",
};

pub const HUD_LABEL_SCORE: LocalizedText = LocalizedText {
    cn: "分数:",
    en: "Scores:",
};

// ==================== 关卡文本（支持变量插值）====================

/// 关卡文本格式化（支持变量插值）
pub const STAGE_TEXT: LocalizedTextFormat = LocalizedTextFormat {
    cn: "第 {var} 关",
    en: "Stage {var}",
};

/// 敌方剩余数量文本格式化（支持变量插值）
pub const ENEMY_COUNT_TEXT: LocalizedTextFormat = LocalizedTextFormat {
    cn: "敌方剩余: {0}/{1}",
    en: "Enemy Left: {0}/{1}",
};
