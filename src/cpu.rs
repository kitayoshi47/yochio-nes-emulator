use crate::bus::Bus; // 先に作ったバスを読み込むよ！

// データの探し方（アドレッシングモード）のリストだよっ♪
#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPage_X,
    ZeroPage_Y,
    Absolute,
    Absolute_X,
    Absolute_Y,
    Indirect_X,
    Indirect_Y,
    NoneAddressing,
}

// ファミコンの頭脳、CPU（6502）の構造体だよっ♪
pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub status: u8,
    pub program_counter: u16,
    pub stack_pointer: u8,
    pub bus: Bus, // CPUの中にバスを繋ぎ込んだよ！
}

impl CPU {
    // 初期化するときに、繋ぎこむバスを渡してあげるんだっ
    pub fn new(bus: Bus) -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: 0,
            program_counter: 0,
            stack_pointer: 0xFD,
            bus,
        }
    }

    // CPUからメモリを読み取るための便利関数だよ
    pub fn mem_read(&self, addr: u16) -> u8 {
        self.bus.mem_read(addr)
    }

    // CPUからメモリに書き込むための便利関数だよ
    pub fn mem_write(&mut self, addr: u16, data: u8) {
        self.bus.mem_write(addr, data)
    }

    // ★ ここからがフェッチ！
    // PCの場所から1バイト読み取って、PCを次へ進めるよ
    fn mem_read_pc(&mut self) -> u8 {
        let data = self.mem_read(self.program_counter);
        self.program_counter += 1;
        data
    }

    // レジスタに値が入ったとき、ゼロフラグとネガティブフラグを更新する便利関数だよっ♪
    fn update_zero_and_negative_flags(&mut self, result: u8) {
        // ゼロフラグ (Z) : ビット1
        if result == 0 {
            // 結果が0なら、ビット1を「1」にする（旗を立てる！）
            // 0b0000_0010 と OR演算(|) をすると、そこだけ1になるんだよ
            self.status = self.status | 0b0000_0010;
        } else {
            // 結果が0じゃないなら、ビット1を「0」にする（旗を下ろす！）
            // 0b1111_1101 と AND演算(&) をすると、そこだけ0になるんだ
            self.status = self.status & 0b1111_1101;
        }

        // ネガティブフラグ (N) : ビット7
        // 結果のビット7が「1」かどうかをチェックするよ
        if result & 0b1000_0000 != 0 {
            // ビット7が1なら、ステータスのビット7も「1」にする
            self.status = self.status | 0b1000_0000;
        } else {
            // ビット7が0なら、ステータスのビット7も「0」にする
            self.status = self.status & 0b0111_1111;
        }
    }

    // プログラムをメモリにガチャン！と読み込んで、実行する便利関数だよっ♪
    // 引数の `program: &[u8]` は「バイトデータのスライス（配列の参照）」っていう
    // Rustならではの書き方で、メモリを節約できる優れものなんだっ！
    pub fn load_and_run(&mut self, program: &[u8]) {
        // プログラムの配列を最初から順番に取り出して、メモリの 0x0000 から書き込んでいくよ
        for (i, &byte) in program.iter().enumerate() {
            self.mem_write(0x0000 + i as u16, byte);
        }

        // スタート地点 (0x0000) にプログラムカウンタをセット！
        self.program_counter = 0x0000;

        // いざ、実行！！
        self.run();
    }

    // 16ビット（2バイト）のデータをリトルエンディアンで読み込む便利関数だよっ♪
    pub fn mem_read_u16(&self, pos: u16) -> u16 {
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos + 1) as u16;
        // 上位バイトを左に8ビットずらして（シフト）、下位バイトとガッチャンコ！
        (hi << 8) | (lo as u16)
    }

    // アドレッシングモードに合わせて、本当のデータの「場所（アドレス）」を計算する魔法の関数！
    pub fn get_operand_address(&self, mode: &AddressingMode) -> u16 {
        match mode {
            // 即値モード：PCが指している場所そのものがデータ！
            AddressingMode::Immediate => self.program_counter,

            // ゼロページモード：PCが指している場所にある1バイトが、そのままゼロページの住所！
            AddressingMode::ZeroPage  => self.mem_read(self.program_counter) as u16,
            
            // アブソリュートモード：PCが指している場所から2バイト読んだのが住所！
            AddressingMode::Absolute => self.mem_read_u16(self.program_counter),
          
            // ゼロページXモード：ゼロページの住所にXレジスタを足すよ（255を超えたら0に戻る！）
            AddressingMode::ZeroPage_X => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_x) as u16;
                addr
            }
            AddressingMode::ZeroPage_Y => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_y) as u16;
                addr
            }

            // アブソリュートXモード：16ビットの住所にXレジスタを足すよ！
            AddressingMode::Absolute_X => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_x as u16);
                addr
            }
            AddressingMode::Absolute_Y => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_y as u16);
                addr
            }

            // インダイレクトX・Y（ポインタのポインタみたいな複雑なやつ！）
            AddressingMode::Indirect_X => {
                let base = self.mem_read(self.program_counter);
                let ptr: u8 = (base as u8).wrapping_add(self.register_x);
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                (hi as u16) << 8 | (lo as u16)
            }
            AddressingMode::Indirect_Y => {
                let base = self.mem_read(self.program_counter);
                let lo = self.mem_read(base as u16);
                let hi = self.mem_read((base as u8).wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | (lo as u16);
                let deref = deref_base.wrapping_add(self.register_y as u16);
                deref
            }

            // アドレス指定を使わない命令のとき
            AddressingMode::NoneAddressing => {
                panic!("アドレス指定がないモードでアドレスを計算しようとしたよ！");
            }
        }
    }

    // LDA命令の本体だよっ♪ 
    // どんな探し方（モード）を渡されても、これ1つで完璧に対応できちゃうの！
    fn lda(&mut self, mode: &AddressingMode) {
        // 1. まず、指定されたモードで「本当の住所」を計算してもらう！
        let addr = self.get_operand_address(mode);
        
        // 2. その住所からデータを読み取る！
        let value = self.mem_read(addr);
        
        // 3. Aレジスタに入れて、フラグを更新！
        self.register_a = value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    // ★ そしてこれがデコード＆エグゼキュートのメインループ！
    pub fn run(&mut self) {
        loop {
            // 1. フェッチ：命令（オペコード）を取ってくる
            let opscode = self.mem_read_pc();

            // 2 & 3. デコード＆エグゼキュート：命令ごとに処理を分けるよ！
            match opscode {
                // 命令 0xA9: LDA (即値モード)
                0xA9 => {
                    self.lda(&AddressingMode::Immediate);
                    self.program_counter += 1; // データ（1バイト）を読んだ分だけPCを進めるよ
                    println!("【LDA】Aレジスタに即値を入れたよっ！(A = 0x{:02x})", self.register_a);
                }

                // ★ 新規追加！ 命令 0xA5: LDA (ゼロページモード)
                0xA5 => {
                    // 引数のモードを「ZeroPage」に変えるだけ！魔法みたいでしょっ？
                    self.lda(&AddressingMode::ZeroPage);
                    self.program_counter += 1; // ゼロページも住所指定に1バイト使うから、PCを1進めるよ
                    println!("【LDA】ゼロページからAレジスタに入れたよっ！(A = 0x{:02x})", self.register_a);
                }

                // 命令 0xAA: TAX (Transfer A to X)
                0xAA => {
                    self.register_x = self.register_a;
                    // ★ こっちもコピーした後にフラグを更新！
                    self.update_zero_and_negative_flags(self.register_x);
                    println!("【TAX】Aの値をXにコピーしたよ！(X = 0x{:02x})", self.register_x);
                }

                // 命令 0x00: BRK (Break)
                // プログラムを終了（強制割り込み）させる命令だよ！
                0x00 => {
                    println!("【BRK】プログラム終了命令が来たから、ループを抜けるねっ。");
                    return; 
                }

                // 命令 0xE8: INX (Increment X Register)
                // Xレジスタの値を1増やすよ！
                0xE8 => {
                    // wrapping_add で安全に +1 するのがRust流だよっ♪
                    self.register_x = self.register_x.wrapping_add(1);
                    
                    // 値が変わったから、ゼロフラグとネガティブフラグも更新！
                    self.update_zero_and_negative_flags(self.register_x);
                    
                    println!("【INX】Xレジスタを1増やしたよ！(X = 0x{:02x})", self.register_x);
                }

                // まだ作ってない命令が来たら、とりあえず止めるね
                _ => {
                    println!("知らない命令（0x{:02x}）が来たからパニックだよ〜！", opscode);
                    break;
                }
            }
        }
    }
}
