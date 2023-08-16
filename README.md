# Internet Computer developer portal の DeepL 自動翻訳
DeepL で [portal](https://github.com/dfinity/portal) の markdown を日本語へ自動翻訳します。
原文と翻訳が並んで出力されます（翻訳の質が悪い場合に、原文を読めるように）。

# 前提条件
## 前提条件１
```
$ rustc --version
rustc 1.67.0 (fc594f156 2023-01-24)
```
バージョンは、いくつでも動くかもしれない。少なくとも、上記バージョンで動作する。

## 前提条件２
[portal](https://github.com/dfinity/portal) の README に従って、npm start してブラウザで http://localhost:3000/ を確認できる状態であること。
```
$ git clone git@github.com:dfinity/portal.git
$ cd portal/
$ git submodule update --init
$ npm install
$ npm start
```

# 実行手順
1. git clone
```
$ git clone git@github.com:tokuryoo/deepl_markdown_translator_to_jp
```

2. main.rs の下記の箇所に、翻訳したいファイルの全体パスを記述する。
```
let path = 
```

3. 無償の DeepL API を利用したい場合
main.rs を修正する。
```
    let url: &str = "https://api-free.deepl.com/v2/translate";
    // let url: &str = "https://api.deepl.com/v2/translate";
```

3. 実行
```
$ cd deepl_markdown_translator_to_jp
$ export AUTH_KEY=<あなたのAPIキー>
$ cargo run
```

# 実行結果
例えば let path で portal/docs/motoko/intro/01-Comments.md を指定した場合、portal/docs/motoko/intro/01-Comments-translated.md が出力される。

# TODO
- [ ] 現状 let path で翻訳したいファイルを指定している。つまり、１度に１ファイルだけしか翻訳できない。何らかの方法で複数のファイルを指定できるようにしたい。自動的に全ファイルを翻訳すると何かと支障がありそう。既に翻訳済みのファイルを再度翻訳することは避けたい（DeepL API の費用削減）。
- [ ] 翻訳結果を xxx-translated.md として別ファイルへ出力ではなく、原文のファイルへ上書きしたい。しかし、動作確認時に繰り返し実行したい場合、不便ではあるので後回しにしている。
- [ ] 翻訳結果が英語のままだったり、そもそも Deepl API を呼び出せてないバグがある。
- [ ] DeeL API の辞書を利用している。"Internet Computer" のような固有名詞は、翻訳結果も "Internet Computer" としたい。しかし、そのように辞書を登録しても無視されてしまう。カタカナへ翻訳してから、英語へ置換すると良いかも？