package org.code.protocol;

public abstract class JavabuilderRuntimeException extends RuntimeException {
  private String fallbackMessage;

  protected JavabuilderRuntimeException(Enum key) { super(key.toString()); }
  protected JavabuilderRuntimeException(Enum key, Throwable cause) { super(key.toString(), cause); }
  protected JavabuilderRuntimeException(Enum key, String fallbackMessage) {
    super(key.toString());
    this.fallbackMessage = fallbackMessage;
  }

  public void setFallbackMessage(String fallbackMessage) { this.fallbackMessage = fallbackMessage; }
  public String getFallbackMessage() { return this.fallbackMessage; }
}
