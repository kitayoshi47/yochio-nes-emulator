// ファミコンのメモリマップを管理する構造体だよっ♪
pub struct Bus {
    // 2KBのWRAM（ワークRAM）
    // 配列のサイズを固定できるから、Rustならメモリ安全に管理できるんだ！
    cpu_vram: [u8; 2048],
}

impl Bus {
    // Busを新しく作るための初期化関数だよ
    pub fn new() -> Self {
        Bus {
            cpu_vram: [0; 2048], // 最初は全部0で埋めておくね
        }
    }

    // CPUがメモリからデータを読み込む時の処理
    pub fn mem_read(&self, addr: u16) -> u8 {
        match addr {
            // WRAMとそのミラー領域 (0x0000 - 0x1FFF)
            0x0000..=0x1FFF => {
                // 0x07FF (10進数で2047) でビット演算（AND）することで、
                // 0x0800以降のアクセスも自動的に0x0000〜0x07FFに丸め込まれるんだ！
                let mirror_down_addr = addr & 0x07FF;
                self.cpu_vram[mirror_down_addr as usize]
            }
            // PPUレジスタとそのミラー (0x2000 - 0x3FFF)
            0x2000..=0x3FFF => {
                let mirror_down_addr = addr & 0x2007;
                // TODO: PPUの読み込み処理は、PPUを作った時に繋ぎこむよ！
                // 今は仮に0を返しておくね。
                0
            }
            // それ以外の未実装領域は、とりあえず0を返してエラーを防ぐよ
            _ => {
                println!("未実装のメモリ読み込みだよ: 0x{:04x}", addr);
                0
            }
        }
    }

    // CPUがメモリにデータを書き込む時の処理
    pub fn mem_write(&mut self, addr: u16, data: u8) {
        match addr {
            // WRAMとそのミラー領域
            0x0000..=0x1FFF => {
                let mirror_down_addr = addr & 0x07FF;
                self.cpu_vram[mirror_down_addr as usize] = data;
            }
            // PPUレジスタとそのミラー
            0x2000..=0x3FFF => {
                let mirror_down_addr = addr & 0x2007;
                // TODO: PPUへの書き込み処理をあとで作るよ！
            }
            // それ以外の領域
            _ => {
                println!("未実装のメモリ書き込みだよ: 0x{:04x} に {}", addr, data);
            }
        }
    }
}
