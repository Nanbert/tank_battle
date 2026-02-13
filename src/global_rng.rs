//! 全局随机数生成器模块
//!
//! 提供统一的随机数生成器资源，支持固定种子

use bevy::prelude::*;
use rand::rngs::StdRng;
use rand::SeedableRng;

/// 全局随机数生成器资源
#[derive(Resource)]
pub struct GlobalRng(StdRng);

impl GlobalRng {
    /// 使用固定种子创建随机数生成器
    pub fn with_seed(seed: u64) -> Self {
        Self(StdRng::seed_from_u64(seed))
    }

    /// 使用系统时间作为种子创建随机数生成器
    pub fn with_system_time() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            // WebAssembly 不支持 SystemTime，使用固定种子
            Self::with_seed(42)
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            use std::time::{SystemTime, UNIX_EPOCH};
            let seed = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            Self::with_seed(seed)
        }
    }

    /// 生成范围内的随机整数
    pub fn gen_range(&mut self, range: std::ops::Range<usize>) -> usize {
        use rand::Rng;
        self.0.gen_range(range)
    }

    /// 生成范围内的随机浮点数
    pub fn gen_range_f32(&mut self, range: std::ops::Range<f32>) -> f32 {
        use rand::Rng;
        self.0.gen_range(range)
    }

    /// 生成随机布尔值
    pub fn gen_bool(&mut self) -> bool {
        use rand::Rng;
        rand::random::<bool>()
    }
}

impl Default for GlobalRng {
    fn default() -> Self {
        Self::with_system_time()
    }
}

/// 插件：注册全局随机数生成器
pub struct GlobalRngPlugin {
    /// 种子，None 表示使用默认固定种子
    pub seed: Option<u64>,
}

impl Default for GlobalRngPlugin {
    fn default() -> Self {
        Self { seed: None }
    }
}

impl Plugin for GlobalRngPlugin {
    fn build(&self, app: &mut App) {
        let rng = match self.seed {
            Some(seed) => GlobalRng::with_seed(seed),
            None => GlobalRng::default(),
        };
        app.insert_resource(rng);
    }
}