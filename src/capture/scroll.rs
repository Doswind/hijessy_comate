//! 长截图（滚动拼接，基础版）。
//!
//! 采用"手动滚动 + 自动拼接"：外部按固定步骤抓取选区帧，
//! 本模块通过对相邻帧重叠条带做竖直方向匹配估算滚动位移并拼接成长图。
//! 重叠匹配失败（内容突变）时回退为整帧堆叠。
//!
//! 注：迭代二浮层暂未接入长截图入口，此模块为已测试的能力预留，后续里程碑接入。
#![allow(dead_code)]
use image::RgbaImage;

/// 模板条带高度（像素）。
const TEMPLATE_H: u32 = 40;
/// 列采样步长（加速匹配）。
const COL_STEP: u32 = 4;
/// 接受匹配的平均通道误差阈值。
const ACCEPT_AVG_ERR: f64 = 26.0;

/// 滚动截图拼接器。
#[derive(Default)]
pub struct ScrollStitcher {
    canvas: Option<RgbaImage>,
    frames: u32,
}

impl ScrollStitcher {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn frames(&self) -> u32 {
        self.frames
    }

    pub fn height(&self) -> u32 {
        self.canvas.as_ref().map_or(0, |c| c.height())
    }

    /// 追加一帧。
    pub fn push(&mut self, frame: RgbaImage) {
        self.frames += 1;
        match self.canvas.take() {
            None => self.canvas = Some(frame),
            Some(canvas) => self.canvas = Some(stitch(canvas, frame)),
        }
    }

    /// 取出拼接结果。
    pub fn result(self) -> Option<RgbaImage> {
        self.canvas
    }
}

/// 将 `frame` 拼接到 `canvas` 下方。
fn stitch(canvas: RgbaImage, frame: RgbaImage) -> RgbaImage {
    let cw = canvas.width();
    let ch = canvas.height();
    let fw = frame.width();
    let fh = frame.height();

    // 宽度不一致或帧过小，直接堆叠。
    if cw != fw || fh <= TEMPLATE_H || ch < TEMPLATE_H {
        return stack(canvas, &frame);
    }

    // 在新帧中搜索与 canvas 底部条带最匹配的位置。
    let mut best_m = 0u32;
    let mut best_err = f64::MAX;
    let max_m = fh - TEMPLATE_H;
    for m in 0..=max_m {
        let err = strip_error(&canvas, &frame, m, cw);
        if err < best_err {
            best_err = err;
            best_m = m;
        }
    }

    if best_err <= ACCEPT_AVG_ERR {
        let new_start = best_m + TEMPLATE_H;
        if new_start >= fh {
            // 无新增内容。
            return canvas;
        }
        append_rows(canvas, &frame, new_start)
    } else {
        // 内容突变，回退堆叠。
        stack(canvas, &frame)
    }
}

/// 计算 canvas 底部 TEMPLATE_H 行与 frame 第 m 行起的平均通道误差。
fn strip_error(canvas: &RgbaImage, frame: &RgbaImage, m: u32, cw: u32) -> f64 {
    let ch = canvas.height();
    let mut sum: u64 = 0;
    let mut count: u64 = 0;
    for row in 0..TEMPLATE_H {
        let cy = ch - TEMPLATE_H + row;
        let fy = m + row;
        let mut x = 0;
        while x < cw {
            let cp = canvas.get_pixel(x, cy);
            let fp = frame.get_pixel(x, fy);
            sum += (cp[0] as i32 - fp[0] as i32).unsigned_abs() as u64;
            sum += (cp[1] as i32 - fp[1] as i32).unsigned_abs() as u64;
            sum += (cp[2] as i32 - fp[2] as i32).unsigned_abs() as u64;
            count += 3;
            x += COL_STEP;
        }
    }
    if count == 0 {
        f64::MAX
    } else {
        sum as f64 / count as f64
    }
}

/// 把 frame 从 `start_row` 起的部分追加到 canvas 下方。
fn append_rows(canvas: RgbaImage, frame: &RgbaImage, start_row: u32) -> RgbaImage {
    let cw = canvas.width();
    let ch = canvas.height();
    let add = frame.height() - start_row;
    let mut out = RgbaImage::new(cw, ch + add);
    // 复制 canvas。
    for y in 0..ch {
        for x in 0..cw {
            out.put_pixel(x, y, *canvas.get_pixel(x, y));
        }
    }
    // 复制新增行。
    for r in 0..add {
        for x in 0..cw {
            out.put_pixel(x, ch + r, *frame.get_pixel(x, start_row + r));
        }
    }
    out
}

/// 整帧堆叠（回退）。
fn stack(canvas: RgbaImage, frame: &RgbaImage) -> RgbaImage {
    let cw = canvas.width();
    let ch = canvas.height();
    let fw = frame.width();
    let fh = frame.height();
    let out_w = cw.max(fw);
    let mut out = RgbaImage::new(out_w, ch + fh);
    for y in 0..ch {
        for x in 0..cw {
            out.put_pixel(x, y, *canvas.get_pixel(x, y));
        }
    }
    for y in 0..fh {
        for x in 0..fw {
            out.put_pixel(x, ch + y, *frame.get_pixel(x, y));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    /// 生成一张“每行由行号编码”的图像的窗口 [top, top+h)。
    fn window(top: u32, h: u32, w: u32) -> RgbaImage {
        let mut img = RgbaImage::new(w, h);
        for y in 0..h {
            let g = ((top + y) % 256) as u8;
            for x in 0..w {
                img.put_pixel(x, y, Rgba([g, (g / 2), (255 - g), 255]));
            }
        }
        img
    }

    #[test]
    fn stitch_overlapping_windows() {
        let w = 12;
        let f1 = window(0, 100, w); // 全局 0..100
        let f2 = window(40, 100, w); // 向下滚动 40 -> 全局 40..140
        let mut st = ScrollStitcher::new();
        st.push(f1);
        st.push(f2);
        let out = st.result().unwrap();
        // 期望拼接为全局 0..140，高度 140。
        assert_eq!(out.height(), 140, "拼接高度应为 140，实际 {}", out.height());
        // 校验若干行内容与全局行号一致。
        for &y in &[0u32, 50, 99, 120, 139] {
            let g = (y % 256) as u8;
            assert_eq!(out.get_pixel(0, y)[0], g, "第 {y} 行内容不匹配");
        }
    }

    #[test]
    fn stitch_disjoint_falls_back_to_stack() {
        let w = 12;
        let f1 = window(0, 60, w);
        // 纯色帧：与 f1 任意条带都不匹配，应触发回退堆叠。
        let mut f2 = RgbaImage::new(w, 60);
        for p in f2.pixels_mut() {
            *p = Rgba([200, 10, 10, 255]);
        }
        let mut st = ScrollStitcher::new();
        st.push(f1);
        st.push(f2);
        let out = st.result().unwrap();
        assert_eq!(out.height(), 120, "回退堆叠高度应为 120");
    }
}
