## これは何？
AHCのテストケースを並列実行します。

## 使い方
### バッチ処理の場合
`settings.toml`に各種設定を書きます。

- in_dir 入力のファイルのディレクトリ
- out_dir 出力を保存するディレクトリ
- exec_command 採点したいプログラムの実行コマンド
- judge_command 採点を行うコマンド

ジャッジのスコアは`Score = 123`のような形で出力されることを想定しています。

実行します。

```bash
cargo run -r --bin batch
```

