package org.code.protocol;
import java.io.File;
/** The session singletons the libraries reach for. The output adapter is set by
 *  the harness so a Painter's actions reach the validation trackers. */
public class GlobalProtocol {
  private OutputAdapter outputAdapter = message -> {};
  private final ContentManager contentManager =
      filename -> new File(System.getProperty("caturra.assets", "."), filename).toURI().toString();
  public void setOutputAdapter(OutputAdapter adapter) { this.outputAdapter = adapter; }
  public OutputAdapter getOutputAdapter() { return this.outputAdapter; }
  public ContentManager getContentManager() { return this.contentManager; }
}
