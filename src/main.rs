extern crate serde;
use serde::Deserialize;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{self, BufRead, Write};
use std::path::Path;

#[derive(Deserialize, Debug)]
struct TranslationResponse {
    translations: Vec<Translation>,
}

#[derive(Deserialize, Debug)]
struct Translation {
    text: String,
}

fn main() {
    // マークダウンをパースしてから DeepL API で翻訳すると、文がバラバラに分解されてしまい、意味の通った翻訳にならないため、あえてパースせずに、DeppL API に渡している。

    // DeepL の挙動は動かしてみないと、わからない。翻訳後、マークダウンの予約語が消えてしまう、インデントが削られる、日本語に翻訳されない、といった事になっていないか、動作確認しながら解消する方法を探りながら実装している。

    // 英語と日本語を並べて出力している。並べて出力するにあたり、見た目が崩れないようにする必要がある。崩れている事に気づいたら、都度、実装を修正している。

    // 上記で苦労しているところ、OpenAI の API だと、どうなのか。苦労せずに実装できる可能性がある？

    let path = "/Users/tokuryo/dfinity/github/portal/docs/motoko/main/about-this-guide.md";
    let path = "/Users/tokuryo/dfinity/github/portal/docs/motoko/main/base-intro.md";
    let path = "/Users/tokuryo/dfinity/github/portal/docs/motoko/main/actor-classes.md";
    let path = "/Users/tokuryo/dfinity/github/portal/docs/concepts/what-is-IC.md";
    let path = "/Users/tokuryo/dfinity/github/portal/docs/developer-docs/gas-cost.md";
    let path =
        "/Users/tokuryo/dfinity/github/portal/docs/developer-docs/frontend/custom-frontend.md";

    // バグ：dfx build hello_backend や npm install --save react react-router-dom 等が２回出力されている。
    let path = "/Users/tokuryo/dfinity/github/portal/docs/developer-docs/frontend/index.md";

    // バグ？：getTokenIdsForUserDip721 を翻訳すると、getTokenIdsForUserDip721 になる。つまり、同じものが２行出力されてしまう。
    let path = "/Users/tokuryo/dfinity/github/portal/docs/samples/nft.md";

    // バグ１：上記と同様のバグ？あり
    // バグ２：最後の行に "-->" と出力されてしまう。markdown のコメントアウト。
    let path = "/Users/tokuryo/dfinity/github/ICRC/ICRCs/ICRC-7/ICRC-7.md";

    // 長文。費用に注意。
    // let path = "/Users/tokuryo/dfinity/github/portal/submodules/interface-spec/spec/index.md";

    println!("path: {path}");
    let input_path = Path::new(path);
    let input_file = File::open(input_path).unwrap();
    let reader = io::BufReader::new(input_file);

    // TODO 入力チェック
    let auth_key = std::env::var("AUTH_KEY").expect("AUTH_KEY must be set");
    let glossary_id = glossary(&auth_key);

    let input_path = Path::new(path);
    let output_path = input_path.with_extension("");
    let output_path = output_path.to_str().unwrap().to_owned() + "-translated.md";
    let mut output_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&output_path)
        .unwrap();

    let mut buffer = String::new();
    let mut is_code_block = false;
    let mut line_number = 0;
    let mut line: String = String::new();
    let mut prev_line: String = String::new();
    let mut prevprev_line: String;
    for (index, current_line) in reader.lines().enumerate() {
        prevprev_line = prev_line;
        prev_line = line;
        line = current_line.unwrap();
        line_number = index + 1;

        // テスト内容に応じて、最初の数行、スキップ
        // if line_number < 87 {
        //     continue;
        // }
        // テスト内容に応じて、中断
        // if line_number == 120 {
        //     break;
        // }

        println!("****** {line_number} ******");
        println!("Line {line_number} '{line}'");

        if line_number == 1 && line.starts_with("import") {
            writeln!(output_file, "{line}").unwrap();
            continue;
        }

        if line.trim() != ":::" {
            writeln!(output_file, "{line}").unwrap();
        }

        if line.trim().starts_with("!") {
            continue;
        }

        if line.trim() == "---" || line.trim().starts_with("sidebar_position") {
            continue;
        }

        // 以降、翻訳不要と判断したケースが続く。

        if line.starts_with("    ")
            && line.chars().nth(4).map_or(false, |c| c != ' ')
            && !(prev_line.trim().starts_with("-")
                || (prev_line.trim().is_empty() && prevprev_line.trim().starts_with("-")))
            && (prev_line.starts_with("        ")
                || (prev_line.trim().is_empty() && prevprev_line.trim().starts_with("        ")))
        {
            println!("DEBUG-space1 {line}");
            continue;
        }
        println!("DEBUG-space2 {line}");
        // - の中の - は考慮しない実装とした
        if line.starts_with("        ")
        // && line.chars().nth(8).map_or(false, |c| c != ' ')
        // && (prev_line.starts_with("    -")
        //     || (prev_line.trim().is_empty() && prevprev_line.starts_with("    -")))
        {
            continue;
        }

        if line.trim().starts_with("|--") || line.trim().starts_with("| --") {
            continue;
        }
        if line.trim().starts_with(":::note")
            || line.trim().starts_with(":::info")
            || line.trim().starts_with(":::caution")
        {
            continue;
        }
        if line.trim().starts_with("```") {
            is_code_block = !is_code_block;
            if !is_code_block {
                continue;
            }
        }

        if line.trim().starts_with("-") && !is_code_block {
            let original_len = line.len();
            println!("original_len: {original_len}");
            let line_trimed = line.trim_start();
            println!("line_trimed.len(): {}", line_trimed.len());
            let space_size = original_len - line_trimed.len();
            let line = line.trim();
            let translation = call_deepl_api(&glossary_id, &auth_key, &line);
            match translation {
                Ok(translation) => {
                    let translation = translation.trim();
                    println!("Translation-: DEBUG translation '{translation}'");
                    // .len() では、文字数ではなくバイト数であるため、.chars().count() とした。
                    let new_translation = format!(
                        "{:>1$}",
                        translation,
                        translation.chars().count() + space_size
                    );
                    println!("Translation-: new_translation '{new_translation}'");
                    writeln!(output_file, "{new_translation}").unwrap();
                    continue;
                }
                Err(e) => {
                    eprintln!("Error occurred: {e}");
                    return;
                }
            }
        }

        if line.trim().starts_with("#") && !is_code_block {
            // たまに翻訳結果も英語になることがあるので、# を取り除いて、# を付け直している
            // "#" または "##" または "###" または "####" の数を取得
            let hash_count = line.chars().take_while(|&c| c == '#').count();
            // "#" を取り除いたテキストを取得
            let clean_text = line.trim_start_matches('#').trim();
            // TODO # 取り除いても効果なし？ ## Pervasive concepts が翻訳されない。
            // 実行結果 意図通り取り除かれてはいる clean_text Pervasive concepts
            println!("clean_text {clean_text}");
            let translation = call_deepl_api(&glossary_id, &auth_key, &clean_text);
            match translation {
                Ok(translation) => {
                    let translation = format!("{} {}", "#".repeat(hash_count), translation);
                    println!("TranslationX: {}", translation);
                    writeln!(output_file, "{translation}").unwrap();
                    writeln!(output_file).unwrap();
                    continue;
                }
                Err(e) => {
                    eprintln!("Error occurred: {}", e);
                    return;
                }
            }
        }
        if line.trim().starts_with("|") && !is_code_block {
            let splited = line.split("|");
            let mut translations = String::new();
            for s in splited {
                let translation = call_deepl_api(&glossary_id, &auth_key, s);
                match translation {
                    Ok(translation) => {
                        translations.push_str(&translation);
                        translations.push_str("|");
                    }
                    Err(e) => {}
                }
            }
            if !translations.is_empty() {
                translations.pop();
            }
            println!("★★★translations: {}", translations);
            writeln!(output_file, "{translations}").unwrap();
            continue;
        }

        if line.trim().is_empty() || is_code_block || line.trim() == ":::" {
            if !buffer.is_empty() {
                let translation = translate_buffer(
                    &glossary_id,
                    &auth_key,
                    &buffer,
                    line_number - buffer.lines().count(),
                );
                buffer.clear();
                match translation {
                    Ok(translation) => {
                        println!("TranslationZ: {}", translation);
                        if line.trim() == ":::" {
                            writeln!(output_file).unwrap();
                            writeln!(output_file, "{translation}").unwrap();
                            writeln!(output_file, ":::").unwrap();
                        } else {
                            writeln!(output_file, "{translation}").unwrap();
                            writeln!(output_file).unwrap();
                        }
                    }
                    Err(e) => {
                        eprintln!("Error occurred: {}", e);
                        return;
                    }
                }
            } else if line.trim() == ":::" {
                writeln!(output_file, ":::").unwrap();
            }
            continue;
        }

        println!("buffer.push_str {}", line);
        buffer.push_str(&line);
        buffer.push(' ');
    }

    if !buffer.is_empty() {
        match translate_buffer(
            &glossary_id,
            &auth_key,
            &buffer,
            line_number - buffer.lines().count() + 1,
        ) {
            Ok(translation) => {
                println!("TranslationA: {}", translation);
                writeln!(output_file).unwrap();
                writeln!(output_file, "{}", translation).unwrap();
                writeln!(output_file).unwrap();
            }
            Err(e) => {
                eprintln!("Error occurred: {}", e);
                return;
            }
        }
    }
    println!("終了。出力先: {output_path}");
}

fn translate_buffer(
    glossary_id: &str,
    auth_key: &str,
    buffer: &str,
    start_line_number: usize,
) -> Result<String, Box<dyn std::error::Error>> {
    match call_deepl_api(glossary_id, auth_key, buffer) {
        Ok(translation) => Ok(translation),
        Err(err) => {
            eprintln!(
                "Failed to translate lines starting from line {}",
                start_line_number
            );
            Err(err)
        }
    }
}

fn call_deepl_api(
    glossary_id: &str,
    auth_key: &str,
    input: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // let url: &str = "https://api-free.deepl.com/v2/translate";
    let url: &str = "https://api.deepl.com/v2/translate";
    // 翻訳後、スペースが削除されてしまうことが多いため
    let leading_spaces = input.chars().take_while(|&c| c == ' ').count();
    // 翻訳後、スペースが削除されているはずだが、そうではなかった場合に備えて
    let input = input.trim();
    // 文の前後に \u{E000} を付けると、markdown の予約語が含まれていても、いい感じに翻訳される傾向にある。
    let input = input.replace(".", ".\u{E000}");
    let input = input.replace("\u{E000}\u{E000}", "\u{E000}");
    let modified_input = format!("{}{}{}", "\u{E000} ", input, " \u{E000}");
    let params: [(&str, &str); 5] = [
        ("glossary_id", glossary_id),
        ("auth_key", auth_key),
        ("text", &modified_input),
        ("source_lang", "en"),
        ("target_lang", "ja"),
    ];

    let client = reqwest::blocking::Client::new();
    let res = client.post(url).form(&params).send()?;

    if res.status().is_success() {
        let mut translation: TranslationResponse = res.json()?;
        translation.translations[0].text = translation.translations[0].text.replace("\u{E000}", "");
        let returned_str = translation.translations[0].text.trim();
        let returned_str = format!(
            "{:>1$}",
            returned_str,
            returned_str.chars().count() + leading_spaces
        );
        Ok(returned_str)
    } else {
        eprintln!("Failed to translate text. Response: {:?}", res.text()?);
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to translate text",
        )))
    }
}

#[derive(serde::Deserialize)]
struct GlossaryResponse {
    glossary_id: String,
    ready: bool,
    name: String,
    source_lang: String,
    target_lang: String,
    creation_time: String,
    entry_count: u32,
}

fn glossary(auth_key: &str) -> String {
    // let url: &str = "https://api-free.deepl.com/v2/glossaries";
    let url: &str = "https://api.deepl.com/v2/glossaries";
    // 辞書
    let entries =
        "canister\tキャニスター\nInternet Computer\tインターネットコンピューター\nminting\t発行\nstandard\t規格\nburn\tバーン\ndapp\tDapp\nArchitecture\tアーキテクチャ\nstate\tステート\nquery call\tクエリーコール\nupdate call\tアップデートコール\nprincipal\tプリンシパル\nactor\tアクター";
    let params: [(&str, &str); 5] = [
        ("name", "internet_computer"),
        ("source_lang", "en"),
        ("target_lang", "ja"),
        ("entries_format", "tsv"),
        ("entries", entries),
    ];

    let client = reqwest::blocking::Client::new();
    let res = client
        .post(url)
        .header("Authorization", format!("DeepL-Auth-Key {}", auth_key))
        .form(&params)
        .send();
    match res {
        Ok(response) => {
            let glossary_response = response.json::<GlossaryResponse>();
            match glossary_response {
                Ok(res) => {
                    println!("glossary_id {}", res.glossary_id);
                    res.glossary_id
                }
                Err(e) => {
                    println!("err {e}");
                    // TODO
                    String::from("err")
                }
            }
        }
        Err(err) => {
            println!("glossary err");
            // TODO
            String::from("err")
        }
    }
}
