//! Removes Whisper repetition loops from a transcription.
//!
//! Whisper sometimes gets stuck and repeats one phrase many times, most often
//! over silence at the end of a recording ("All right. All right. All right.").
//! `collapse_repetition_loops` keeps one copy of the repeated phrase.

/// Longest repeated phrase, in words, that the check looks for.
const MAX_PHRASE_WORDS: usize = 12;
/// A phrase of two or more words is a loop if it occurs this many times in a row.
const MIN_PHRASE_REPEATS: usize = 4;
/// A single word needs more repeats, because people do say "no no no no".
const MIN_WORD_REPEATS: usize = 5;

/// The form of a word used for comparison: lowercase, punctuation removed.
/// "Right." and "right," compare equal. A token with no letters or digits
/// compares as itself.
fn word_key(word: &str) -> String {
    let key: String = word
        .chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect();
    if key.is_empty() {
        word.to_string()
    } else {
        key
    }
}

/// Count how many times the phrase `keys[start..start + len]` occurs in a row,
/// starting at `start`. The first occurrence counts as 1.
fn count_repeats(keys: &[String], start: usize, len: usize) -> usize {
    let phrase = &keys[start..start + len];
    let mut count = 1;
    while keys
        .get(start + count * len..start + (count + 1) * len)
        .is_some_and(|next| next == phrase)
    {
        count += 1;
    }
    count
}

/// Collapse each repetition loop to one copy of its phrase. Returns the text
/// unchanged (apart from single spaces between words) if there is no loop.
pub fn collapse_repetition_loops(text: &str) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    let keys: Vec<String> = words.iter().map(|w| word_key(w)).collect();
    let mut out: Vec<&str> = Vec::with_capacity(words.len());

    let mut i = 0;
    while i < words.len() {
        // Find the shortest phrase that starts here and loops.
        let looped = (1..=MAX_PHRASE_WORDS.min(words.len() - i)).find_map(|len| {
            let repeats = count_repeats(&keys, i, len);
            let needed = if len == 1 { MIN_WORD_REPEATS } else { MIN_PHRASE_REPEATS };
            (repeats >= needed).then_some((len, repeats))
        });

        let Some((len, repeats)) = looped else {
            out.push(words[i]);
            i += 1;
            continue;
        };

        // Keep the first copy and skip the rest.
        out.extend_from_slice(&words[i..i + len]);
        let mut next = i + len * repeats;

        // A loop cut off by the end of the audio leaves a partial copy
        // ("All right. All right. All"). Drop it only if it runs to the end
        // of the text, so real words after a loop are never removed.
        let partial = keys[next..]
            .iter()
            .zip(&keys[i..i + len])
            .take_while(|(a, b)| a == b)
            .count();
        if next + partial == words.len() {
            next += partial;
        }

        i = next;
    }

    out.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapses_a_phrase_loop_at_the_end() {
        let looped = format!("Let's keep it on main.{}", " All right.".repeat(28));
        assert_eq!(
            collapse_repetition_loops(&looped),
            "Let's keep it on main. All right."
        );
    }

    #[test]
    fn collapses_a_loop_without_punctuation() {
        let looped = format!("that is the{}", " end of this".repeat(12));
        assert_eq!(collapse_repetition_loops(&looped), "that is the end of this");
    }

    #[test]
    fn drops_a_partial_copy_at_the_end() {
        let looped = "Done. All right. All right. All right. All right. All";
        assert_eq!(collapse_repetition_loops(looped), "Done. All right.");
    }

    #[test]
    fn keeps_real_words_after_a_loop() {
        let text = "All right. All right. All right. All right. All of this is fine.";
        assert_eq!(
            collapse_repetition_loops(text),
            "All right. All of this is fine."
        );
    }

    #[test]
    fn collapses_a_loop_in_the_middle() {
        let text = "First point. you know you know you know you know Second point.";
        assert_eq!(
            collapse_repetition_loops(text),
            "First point. you know Second point."
        );
    }

    #[test]
    fn ignores_case_and_punctuation_differences() {
        let text = "Okay. Thank you. thank you, Thank you! thank you.";
        assert_eq!(collapse_repetition_loops(text), "Okay. Thank you.");
    }

    #[test]
    fn collapses_a_single_word_loop() {
        assert_eq!(collapse_repetition_loops("so the the the the the the plan"), "so the plan");
    }

    #[test]
    fn leaves_normal_repetition_alone() {
        for text in [
            "No, no, no, no, that is wrong.",
            "It is very, very, very important.",
            "I think that that works. I think that we ship. I think that is all.",
            "All right. All right. All right. Let's go.",
        ] {
            assert_eq!(collapse_repetition_loops(text), text);
        }
    }

    #[test]
    fn handles_empty_and_short_text() {
        assert_eq!(collapse_repetition_loops(""), "");
        assert_eq!(collapse_repetition_loops("Oh, yeah."), "Oh, yeah.");
    }
}
