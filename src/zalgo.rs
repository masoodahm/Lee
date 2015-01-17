use std::rand;
use std::rand::{thread_rng, Rng};

static ZALGO_CHARS: [char; 113]  = [
  '\u{30d}', /*     Ì     */		 '\u{30e}', /*     ÌŽ     */	 '\u{304}', /*     Ì„     */	 '\u{305}', /*     Ì…     */
  '\u{33f}', /*     Ì¿     */		'\u{311}', /*     Ì‘     */		'\u{306}', /*     Ì†     */		'\u{310}', /*     Ì     */
  '\u{352}', /*     Í’     */		'\u{357}', /*     Í—     */		'\u{351}', /*     Í‘     */		'\u{307}', /*     Ì‡     */
  '\u{308}', /*     Ìˆ     */		'\u{30a}', /*     ÌŠ     */		'\u{342}', /*     Í‚     */		'\u{343}', /*     Ì“     */
  '\u{344}', /*     ÌˆÌ     */	 '\u{34a}', /*     ÍŠ     */	 '\u{34b}', /*     Í‹     */	 '\u{34c}', /*     ÍŒ     */
  '\u{303}', /*     Ìƒ     */		'\u{302}', /*     Ì‚     */		'\u{30c}', /*     ÌŒ     */		'\u{350}', /*     Í     */
  '\u{300}', /*     Ì€     */		'\u{301}', /*     Ì     */		 '\u{30b}', /*     Ì‹     */	 '\u{30f}', /*     Ì     */
  '\u{312}', /*     Ì’     */		'\u{313}', /*     Ì“     */		'\u{314}', /*     Ì”     */		'\u{33d}', /*     Ì½     */
  '\u{309}', /*     Ì‰     */		'\u{363}', /*     Í£     */		'\u{364}', /*     Í¤     */		'\u{365}', /*     Í¥     */
  '\u{366}', /*     Í¦     */		'\u{367}', /*     Í§     */		'\u{368}', /*     Í¨     */		'\u{369}', /*     Í©     */
  '\u{36a}', /*     Íª     */		'\u{36b}', /*     Í«     */		'\u{36c}', /*     Í¬     */		'\u{36d}', /*     Í­     */
  '\u{36e}', /*     Í®     */		'\u{36f}', /*     Í¯     */		'\u{33e}', /*     Ì¾     */		'\u{35b}', /*     Í›     */
  '\u{346}', /*     Í†     */		'\u{31a}', /*     Ìš     */
  '\u{316}', /*     Ì–     */		'\u{317}', /*     Ì—     */		'\u{318}', /*     Ì˜     */		'\u{319}', /*     Ì™     */
  '\u{31c}', /*     Ìœ     */		'\u{31d}', /*     Ì     */		 '\u{31e}', /*     Ìž     */	 '\u{31f}', /*     ÌŸ     */
  '\u{320}', /*     Ì      */		'\u{324}', /*     Ì¤     */		'\u{325}', /*     Ì¥     */		'\u{326}', /*     Ì¦     */
  '\u{329}', /*     Ì©     */		'\u{32a}', /*     Ìª     */		'\u{32b}', /*     Ì«     */		'\u{32c}', /*     Ì¬     */
  '\u{32d}', /*     Ì­     */		 '\u{32e}', /*     Ì®     */	 '\u{32f}', /*     Ì¯     */	 '\u{330}', /*     Ì°     */
  '\u{331}', /*     Ì±     */		'\u{332}', /*     Ì²     */		'\u{333}', /*     Ì³     */		'\u{339}', /*     Ì¹     */
  '\u{33a}', /*     Ìº     */		'\u{33b}', /*     Ì»     */		'\u{33c}', /*     Ì¼     */		'\u{345}', /*     Í…     */
  '\u{347}', /*     Í‡     */		'\u{348}', /*     Íˆ     */		'\u{349}', /*     Í‰     */		'\u{34d}', /*     Í     */
  '\u{34e}', /*     ÍŽ     */		'\u{353}', /*     Í“     */		'\u{354}', /*     Í”     */		'\u{355}', /*     Í•     */
  '\u{356}', /*     Í–     */		'\u{359}', /*     Í™     */		'\u{35a}', /*     Íš     */		'\u{323}', /*     Ì£     */
  '\u{315}', /*     Ì•     */		'\u{31b}', /*     Ì›     */		'\u{340}', /*     Ì€     */		'\u{341}', /*     Ì     */
  '\u{358}', /*     Í˜     */		'\u{321}', /*     Ì¡     */		'\u{322}', /*     Ì¢     */		'\u{327}', /*     Ì§     */
  '\u{328}', /*     Ì¨     */		'\u{334}', /*     Ì´     */		'\u{335}', /*     Ìµ     */		'\u{336}', /*     Ì¶     */
  '\u{34f}', /*     Í     */		 '\u{35c}', /*     Íœ     */	 '\u{35d}', /*     Í     */		'\u{35e}', /*     Íž     */
  '\u{35f}', /*     ÍŸ     */		'\u{360}', /*     Í      */		'\u{362}', /*     Í¢     */		'\u{338}', /*     Ì¸     */
  '\u{337}', /*     Ì·     */		'\u{361}', /*     Í¡     */		'\u{489}' /*     Ò‰_     */
];

pub fn make_zalgo(input: String) -> String {
  let mut result: String = String::new();
  for character in input.chars() {
    result.push_str(character.to_string().as_slice());

    if character == ' ' {
      continue;
    }

    for _ in range(0, 5 + (rand::random::<u32>() % 10)) {
      result.push_str(thread_rng().choose(&ZALGO_CHARS).unwrap().to_string().as_slice());
    }
  }

  result
}