package org.json;

import java.util.HashMap;
import java.util.Map;
import java.util.Set;

/** The subset org.code uses: a string/int map, built from a Map or parsed. */
public class JSONObject {
  public static final Object NULL = new Object() {
    @Override public String toString() { return "null"; }
    @Override public boolean equals(Object other) { return other == this || other == null; }
    @Override public int hashCode() { return 0; }
  };

  private final Map<String, Object> entries;

  JSONObject(Map<String, Object> entries) { this.entries = entries; }

  public JSONObject() { this.entries = new HashMap<>(); }

  /** ClientMessage builds its detail from a HashMap<String, String>. */
  public JSONObject(HashMap<String, String> source) {
    this.entries = new HashMap<>();
    if (source != null) {
      for (Map.Entry<String, String> e : source.entrySet()) {
        this.entries.put(e.getKey(), e.getValue());
      }
    }
  }

  public boolean has(String key) { return this.entries.containsKey(key); }

  public int length() { return this.entries.size(); }

  /** The real org.json returns Set<String>; NeighborhoodTracker streams over it. */
  public Set<String> keySet() { return this.entries.keySet(); }

  public boolean isNull(String key) {
    Object value = this.entries.get(key);
    return value == null || NULL.equals(value);
  }

  public Object get(String key) {
    if (!this.entries.containsKey(key)) { throw new JSONException("no such key: " + key); }
    return this.entries.get(key);
  }

  public String getString(String key) { return String.valueOf(get(key)); }

  public int getInt(String key) {
    Object value = get(key);
    if (value instanceof Integer) { return ((Integer) value).intValue(); }
    return Integer.parseInt(String.valueOf(value));
  }

  public boolean getBoolean(String key) {
    Object value = get(key);
    if (value instanceof Boolean) { return ((Boolean) value).booleanValue(); }
    return Boolean.parseBoolean(String.valueOf(value));
  }

  public JSONObject put(String key, Object value) {
    this.entries.put(key, value);
    return this;
  }

  @Override
  public String toString() { return this.entries.toString(); }
}
