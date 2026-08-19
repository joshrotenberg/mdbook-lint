# Unicode Test Document

This document tests various Unicode characters and edge cases.

## Emoji and Symbols

- List with emoji 📝
- Mathematical symbols: ∑ ∫ ∂ ∆ ∞ ≠ ≤ ≥  
- Currency: € £ ¥ ₹ ₽

## International Text

- Chinese: 中文测试内容
- Arabic: اختبار المحتوى العربي
- Hebrew: בדיקת תוכן בעברית
- Russian: Тестирование русского содержания
- Japanese: 日本語のテスト内容

## Special Characters

Zero-width characters: invisible‌spaces\u{200B}here

Right-to-left override: normal text ‮reversed text‬ normal again

## Smart Quotes and Dashes

"Smart quotes" vs "regular quotes"
En dash: – Em dash: — Minus: −

## Code with Unicode

```python  
def greet():
    print("Hello, 世界!")  # Mixed ASCII and Unicode
```

This should test Unicode handling across different rules.