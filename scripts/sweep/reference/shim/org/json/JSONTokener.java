package org.json;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

/** Recursive descent over arrays, objects, strings, ints and null. */
final class JSONTokener {
  private final String text;
  private int pos;

  JSONTokener(String text) { this.text = text; this.pos = 0; }

  Object parse() {
    skipWhitespace();
    Object value = parseValue();
    skipWhitespace();
    if (this.pos != this.text.length()) { throw new JSONException("trailing content"); }
    return value;
  }

  private void skipWhitespace() {
    while (this.pos < this.text.length() && Character.isWhitespace(this.text.charAt(this.pos))) {
      this.pos++;
    }
  }

  private char peek() {
    if (this.pos >= this.text.length()) { throw new JSONException("unexpected end of input"); }
    return this.text.charAt(this.pos);
  }

  private Object parseValue() {
    char c = peek();
    if (c == '[') { return parseArray(); }
    if (c == '{') { return parseObject(); }
    if (c == '"') { return parseString(); }
    return parseLiteral();
  }

  private List<Object> parseArray() {
    List<Object> values = new ArrayList<>();
    this.pos++;
    skipWhitespace();
    if (peek() == ']') { this.pos++; return values; }
    while (true) {
      skipWhitespace();
      values.add(parseValue());
      skipWhitespace();
      char c = peek();
      this.pos++;
      if (c == ']') { return values; }
      if (c != ',') { throw new JSONException("expected , or ]"); }
    }
  }

  private Map<String, Object> parseObject() {
    Map<String, Object> entries = new HashMap<>();
    this.pos++;
    skipWhitespace();
    if (peek() == '}') { this.pos++; return entries; }
    while (true) {
      skipWhitespace();
      String key = parseString();
      skipWhitespace();
      if (peek() != ':') { throw new JSONException("expected :"); }
      this.pos++;
      skipWhitespace();
      entries.put(key, parseValue());
      skipWhitespace();
      char c = peek();
      this.pos++;
      if (c == '}') { return entries; }
      if (c != ',') { throw new JSONException("expected , or }"); }
    }
  }

  private String parseString() {
    if (peek() != '"') { throw new JSONException("expected a string"); }
    this.pos++;
    StringBuilder text = new StringBuilder();
    while (true) {
      char c = peek();
      this.pos++;
      if (c == '"') { return text.toString(); }
      text.append(c);
    }
  }

  private Object parseLiteral() {
    int start = this.pos;
    while (this.pos < this.text.length() && ",]} \t\r\n".indexOf(this.text.charAt(this.pos)) < 0) {
      this.pos++;
    }
    String token = this.text.substring(start, this.pos);
    if (token.equals("null")) { return JSONObject.NULL; }
    if (token.equals("true")) { return Boolean.TRUE; }
    if (token.equals("false")) { return Boolean.FALSE; }
    try {
      return Integer.valueOf(token);
    } catch (NumberFormatException notANumber) {
      throw new JSONException("bad literal '" + token + "'");
    }
  }
}
