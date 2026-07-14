package org.json;

import java.util.List;
import java.util.Map;

public class JSONArray {
  private final List<Object> values;

  @SuppressWarnings("unchecked")
  public JSONArray(String source) {
    Object parsed = new JSONTokener(source).parse();
    if (!(parsed instanceof List)) { throw new JSONException("not a JSON array"); }
    this.values = (List<Object>) parsed;
  }

  JSONArray(List<Object> values) { this.values = values; }

  public int length() { return this.values.size(); }

  @SuppressWarnings("unchecked")
  public Object get(int index) {
    if (index < 0 || index >= this.values.size()) { throw new JSONException("index out of range"); }
    Object value = this.values.get(index);
    if (value instanceof List) { return new JSONArray((List<Object>) value); }
    if (value instanceof Map) { return new JSONObject((Map<String, Object>) value); }
    return value;
  }
}
