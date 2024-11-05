#[derive(Debug)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

pub struct LineGraph {
    points: Vec<Point>,
}

impl LineGraph {
    // 折れ線グラフの定義
    pub fn new(points: Vec<Point>) -> Self {
        LineGraph { points }
    }

    // x座標に対するy座標を求めるメソッド
    pub fn get_y(&self, x: f64) -> Option<f64> {
        // 点が2つ以上あることを確認
        if self.points.len() < 2 {
            return None;
        }

        // x座標が範囲外かチェック
        let (first, last) = (self.points.first().unwrap(), self.points.last().unwrap());
        if x < first.x || x > last.x {
            // 端っこにある点のy座標を返す
            return Some(if x < first.x { first.y } else { last.y });
        }

        // x座標の区間を見つけて線形補間を行う
        for i in 0..self.points.len() - 1 {
            let p1 = &self.points[i];
            let p2 = &self.points[i + 1];

            if x >= p1.x && x <= p2.x {
                // 線形補間の計算
                let t = (x - p1.x) / (p2.x - p1.x);
                return Some(p1.y + t * (p2.y - p1.y));
            }
        }

        None
    }
}
