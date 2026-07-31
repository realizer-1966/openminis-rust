// JavaScript 헬퍼 — 브라우저 자동화에 사용되는 JS 스니펫
// 원본: BrowserUseJS.kt

/// 요소 찾기, 텍스트 추출, 클릭 등의 JS 코드 생성
pub struct Js;

impl Js {
    /// CSS 선택자로 요소 클릭
    pub fn click_selector(selector: &str) -> String {
        format!(
            r#"(function(){{var el=document.querySelector({});if(el){{el.click();return 'ok'}}return 'not_found'}})()"#,
            js_string(selector)
        )
    }

    /// 좌표로 요소 클릭
    pub fn click_coords(x: i32, y: i32) -> String {
        format!(
            r#"(function(){{var el=document.elementFromPoint({},{});if(el){{el.click();return 'ok'}}return 'not_found'}})()"#,
            x, y
        )
    }

    /// 텍스트 입력
    pub fn type_text(selector: &str, text: &str) -> String {
        format!(
            r#"(function(){{var el=document.querySelector({});if(el){{el.focus();el.value={};el.dispatchEvent(new Event('input',{{bubbles:true}}));el.dispatchEvent(new Event('change',{{bubbles:true}}));return 'ok'}}return 'not_found'}})()"#,
            js_string(selector),
            js_string(text)
        )
    }

    /// 요소의 텍스트 추출
    pub fn get_text(selector: &str) -> String {
        format!(
            r#"(function(){{var el=document.querySelector({});return el?el.textContent:null}})()"#,
            js_string(selector)
        )
    }

    /// 페이지 전체 텍스트 추출 (Readability 스타일)
    pub fn get_readable() -> &'static str {
        r#"(function(){
            var article=document.querySelector('article')||document.querySelector('main')||document.body;
            var clone=article.cloneNode(true);
            clone.querySelectorAll('script,style,noscript,svg,nav,footer,header,aside').forEach(function(e){e.remove()});
            return clone.innerText||clone.textContent||'';
        })()"#
    }

    /// 스크롤
    pub fn scroll(direction: &str, amount: i32) -> String {
        let delta = if direction == "up" { -amount } else { amount };
        format!(r#"window.scrollBy(0,{})"#, delta)
    }

    /// 페이지 정보 (URL, 제목)
    pub fn get_page_info() -> &'static str {
        r#"(function(){return JSON.stringify({url:location.href,title:document.title,readyState:document.readyState})})()"#
    }

    /// DOM 구조 (백본) — 간소화된 트리
    pub fn get_backbone(max_depth: i32) -> String {
        format!(
            r#"(function(d,m){{function walk(n,depth){{if(!n||depth>m)return'';var t=n.tagName?n.tagName.toLowerCase():'#text';var s='<'+t;if(n.id)s+=' id="'+n.id+'"';if(n.className&&typeof n.className==='string')s+=' class="'+n.className.split(' ').slice(0,3).join(' ')+'"';s+='>';if(depth<m){{for(var i=0;i<n.children.length;i++)s+=walk(n.children[i],depth+1);}}return s}}return walk(document.body,0)}})(document,{})"#,
            max_depth
        )
    }

    /// find_elements — 선택자에 매칭되는 요소 수 반환
    pub fn find_elements(selector: &str) -> String {
        format!(
            r#"(function(){{var els=document.querySelectorAll({});return els.length}})()"#,
            js_string(selector)
        )
    }

    /// hover
    pub fn hover(selector: &str) -> String {
        format!(
            r#"(function(){{var el=document.querySelector({});if(el){{var e=new MouseEvent('mouseover',{{bubbles:true}});el.dispatchEvent(e);return 'ok'}}return 'not_found'}})()"#,
            js_string(selector)
        )
    }
}

/// JS 문자열 리터럴로 안전하게 인용
fn js_string(s: &str) -> String {
    let escaped = s.replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\n', "\\n")
        .replace('\r', "\\r");
    format!("'{}'", escaped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_click_selector() {
        let js = Js::click_selector("#button");
        assert!(js.contains("querySelector"));
        assert!(js.contains("#button"));
    }

    #[test]
    fn test_click_coords() {
        let js = Js::click_coords(100, 200);
        assert!(js.contains("100"));
        assert!(js.contains("200"));
        assert!(js.contains("elementFromPoint"));
    }

    #[test]
    fn test_type_text() {
        let js = Js::type_text("#input", "hello");
        assert!(js.contains("#input"));
        assert!(js.contains("hello"));
        assert!(js.contains("dispatchEvent"));
    }

    #[test]
    fn test_get_text() {
        let js = Js::get_text(".content");
        assert!(js.contains("textContent"));
    }

    #[test]
    fn test_scroll() {
        let js = Js::scroll("down", 500);
        assert!(js.contains("500"));
        assert!(js.contains("scrollBy"));
    }

    #[test]
    fn test_scroll_up() {
        let js = Js::scroll("up", 300);
        assert!(js.contains("-300"));
    }

    #[test]
    fn test_get_page_info() {
        let js = Js::get_page_info();
        assert!(js.contains("location.href"));
        assert!(js.contains("document.title"));
    }

    #[test]
    fn test_get_readable() {
        let js = Js::get_readable();
        assert!(js.contains("article"));
        assert!(js.contains("innerText"));
    }

    #[test]
    fn test_get_backbone() {
        let js = Js::get_backbone(3);
        assert!(js.contains("3"));
        assert!(js.contains("tagName"));
    }

    #[test]
    fn test_js_string_escape() {
        let s = js_string("it's a test");
        assert!(s.contains("\\'"));
    }

    #[test]
    fn test_find_elements() {
        let js = Js::find_elements("div.card");
        assert!(js.contains("querySelectorAll"));
        assert!(js.contains("div.card"));
    }

    #[test]
    fn test_hover() {
        let js = Js::hover("#menu-item");
        assert!(js.contains("mouseover"));
    }
}