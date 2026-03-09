pub mod bus;
pub mod cpu;

use bus::Bus;
use cpu::CPU;

fn main() {
    let bus = Bus::new();
    let mut cpu = CPU::new(bus);

    println!("さあ、テストプログラムを実行するよー！");
    
    // LDA(0x42) -> TAX -> BRK のプログラムを一気に読み込んで実行！
    cpu.load_and_run(&[0xA9, 0x42, 0xAA, 0x00]);

    println!("実行後のAレジスタ: 0x{:02x}", cpu.register_a);
    println!("実行後のXレジスタ: 0x{:02x}", cpu.register_x);
    println!("実行後のステータス: 0b{:08b}", cpu.status);
}

// --- ここから下はテストコードだよっ♪ ---

#[cfg(test)]
mod test {
    use super::*;
    use crate::bus::Bus;

    // 1つ目のテスト：LDA命令で正しくデータが入るか？
    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        
        // 見て見て！こんなに1行でスッキリ書けるようになったよっ！✨
        // &[0xA9, 0x05, 0x00] がテストプログラムだよ。
        cpu.load_and_run(&[0xA9, 0x05, 0x00]);

        assert_eq!(cpu.register_a, 0x05);
        assert_eq!(cpu.status & 0b1000_0010, 0b0000_0000);
    }

    // 2つ目のテスト：LDA命令でゼロフラグがちゃんと立つか？
    #[test]
    fn test_0xa9_lda_zero_flag() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        
        // こっちも1行で準備完了！
        cpu.load_and_run(&[0xA9, 0x00, 0x00]);

        assert_eq!(cpu.status & 0b0000_0010, 0b0000_0010);
    }

    // 3つ目のテスト：LDA命令でネガティブフラグがちゃんと立つか？
    #[test]
    fn test_0xa9_lda_negative_flag() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        
        // 0x80 は2進数で「1000_0000」。
        // 一番左のビット（ビット7）が1だから、ネガティブフラグが立つはずだよっ！
        cpu.load_and_run(&[0xA9, 0x80, 0x00]);

        // ステータスの「ビット7（ネガティブフラグ）」がちゃんと1になっているかチェック！
        assert_eq!(cpu.status & 0b1000_0000, 0b1000_0000);
    }

    // 4つ目のテスト：TAX命令でAレジスタからXレジスタへ正しくコピーされるか？
    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        
        // LDAでAレジスタに 0x0A を入れてから、TAXでXにコピーするプログラムだよ
        cpu.load_and_run(&[0xA9, 0x0A, 0xAA, 0x00]);

        // Xレジスタにちゃんと 0x0A が入っているかチェック！
        assert_eq!(cpu.register_x, 0x0A);
    }

    // 5つ目のテスト：INX命令でXレジスタが正しく1増えるか？
    #[test]
    fn test_0xe8_inx_increment() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        
        // テストの準備：あらかじめXレジスタに 0x00 を入れておくよ
        cpu.register_x = 0x00;
        
        // INX (0xE8) を実行！
        cpu.load_and_run(&[0xE8, 0x00]);

        // ちゃんと 0x01 に増えているかチェック！
        assert_eq!(cpu.register_x, 0x01);
    }

    // 6つ目のテスト：INX命令でオーバーフロー（255の次は0）が正しく起きるか？
    #[test]
    fn test_0xe8_inx_overflow() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        
        // テストの準備：Xレジスタをギリギリの 255 (0xFF) にしておくよ
        cpu.register_x = 0xFF;
        
        // INX (0xE8) を実行！
        cpu.load_and_run(&[0xE8, 0x00]);
        
        // ぐるっと回って 0x00 に戻っているはず！
        assert_eq!(cpu.register_x, 0x00);
        
        // 結果が 0 になったから、ゼロフラグが立っているかもチェック！
        assert_eq!(cpu.status & 0b0000_0010, 0b0000_0010);
    }

    // 7つ目のテスト：LDAのゼロページモードが正しく動くか？
    #[test]
    fn test_0xa5_lda_zero_page() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);

        // テストの準備：メモリの 0x0010 番地に、あらかじめ 0x55 というデータを置いておくよ！
        cpu.mem_write(0x0010, 0x55);

        // プログラムを実行！
        // 0xA5 (LDA ゼロページモード)
        // 0x10 (読みに行く住所は 0x0010 だよって指定)
        // 0x00 (BRK 終了)
        cpu.load_and_run(&[0xA5, 0x10, 0x00]);

        // Aレジスタに、ちゃんと 0x0010 番地のデータ(0x55) が入っているかチェック！
        assert_eq!(cpu.register_a, 0x55);
    }
}
