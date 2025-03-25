use std::sync::Arc;
use dashmap::DashMap;
use rayon::prelude::*;

// 盤面サイズ：x軸, y軸は 2、z軸は n（ここでは例として n = 5）
const X_DIM: u32 = 2;
const Y_DIM: u32 = 3;
const Z_DIM: u32 = 19; // ここを変えることで n の値を調整可能
const TOT: u32 = X_DIM * Y_DIM * Z_DIM; // 全ブロック数（2*2*n = 4*n）

/// インデックス -> 座標 (x, y, z) への変換
fn index_to_coord(i: u32) -> (u32, u32, u32) {
    let x = i % X_DIM;
    let y = (i / X_DIM) % Y_DIM;
    let z = i / (X_DIM * Y_DIM);
    (x, y, z)
}

/// 座標 a が座標 b 以上か（各成分について a.0>=b.0, a.1>=b.1, a.2>=b.2）
fn coord_ge(a: (u32, u32, u32), b: (u32, u32, u32)) -> bool {
    a.0 >= b.0 && a.1 >= b.1 && a.2 >= b.2
}

/// 選んだ座標 chosen 以上の座標を持つブロック群を取り除くためのマスクを返す
fn removal_mask(chosen: (u32, u32, u32)) -> u128 {
    let mut mask: u128 = 0;
    for i in 0..TOT {
        let coord = index_to_coord(i);
        if coord_ge(coord, chosen) {
            mask |= 1 << i;
        }
    }
    mask
}

/// 現在の状態 state（各ブロックの存在をビットで表現）から、合法な手を返す。
/// 手は (chosen: (x,y,z), new_state: u128) の組として返す。
/// 毒ブロック (0,0,0) は選べない手としています。
fn legal_moves(state: u128) -> Vec<((u32, u32, u32), u128)> {
    let mut moves = Vec::new();
    for i in 0..TOT {
        // ブロック i が存在しているかチェック
        if state & (1 << i) != 0 {
            let chosen = index_to_coord(i);
            if chosen == (0, 0, 0) {
                continue; // 毒ブロックは選べない
            }
            let rm_mask = removal_mask(chosen);
            // もし取り除くブロック群に毒ブロック (0,0,0)（インデックス 0）が含まれていたら不合法
            if rm_mask & 1 != 0 {
                continue;
            }
            let new_state = state & !rm_mask;
            moves.push((chosen, new_state));
        }
    }
    moves
}

/// 現在の状態 state で、手番のプレイヤーが勝てるかどうかを並列再帰的に判定する関数。
/// memo は Arc 化した DashMap を用いて並列安全にメモ化します。
fn win(state: u128, memo: &Arc<DashMap<u128, bool>>) -> bool {
    // 終端状態：毒ブロックのみが残っている場合は負け
    if state == 1 {
        return false;
    }
    if let Some(res) = memo.get(&state) {
        return *res;
    }
    let moves = legal_moves(state);
    if moves.is_empty() {
        memo.insert(state, false);
        return false;
    }
    // 合法手について、Rayon の par_iter() を使って並列に再帰的に評価
    let winning = moves.par_iter().any(|&(_, new_state)| {
        !win(new_state, memo)
    });
    memo.insert(state, winning);
    winning
}

/// 現在の状態から、勝利につながる（必勝となる）手（chosen 座標）の候補をすべて返す
fn winning_moves(state: u128, memo: &Arc<DashMap<u128, bool>>) -> Vec<(u32, u32, u32)> {
    legal_moves(state)
        .into_iter()
        .filter_map(|(mv, new_state)| if !win(new_state, memo) { Some(mv) } else { None })
        .collect()
}

fn main() {
    // 初期状態: 全ブロックが存在している → 下位 TOT ビットがすべて 1
    let initial_state: u128 = (1u128 << TOT) - 1;
    let memo = Arc::new(DashMap::new());
    
    println!("計算開始...");
    let first_win = win(initial_state, &memo);
    println!("初期状態は先手必勝か: {}", first_win);
    
    let moves = winning_moves(initial_state, &memo);
    println!("先手の必勝手候補:");
    for mv in moves {
        println!("{:?}", mv);
    }
}
