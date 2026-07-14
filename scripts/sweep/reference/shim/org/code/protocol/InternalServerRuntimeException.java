package org.code.protocol;
public class InternalServerRuntimeException extends JavabuilderRuntimeException {
  public InternalServerRuntimeException(InternalExceptionKey key) { super(key); }
  public InternalServerRuntimeException(InternalExceptionKey key, Throwable cause) { super(key, cause); }
}
