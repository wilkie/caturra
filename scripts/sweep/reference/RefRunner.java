import java.io.PrintStream;
import java.util.ArrayList;
import java.util.List;
import org.junit.platform.engine.TestExecutionResult;
import org.junit.platform.engine.discovery.ClassSelector;
import org.junit.platform.engine.discovery.DiscoverySelectors;
import org.junit.platform.launcher.Launcher;
import org.junit.platform.launcher.LauncherDiscoveryRequest;
import org.junit.platform.launcher.TestExecutionListener;
import org.junit.platform.launcher.TestIdentifier;
import org.junit.platform.launcher.TestPlan;
import org.junit.platform.launcher.core.LauncherDiscoveryRequestBuilder;
import org.junit.platform.launcher.core.LauncherFactory;

/**
 * The reference grading run: discover and execute the named test classes with
 * REAL JUnit 5, and print one `__VTEST\t<PASS|FAIL>\t<name>\t<message>` line per
 * test — the same format caturra's synthesized __ValidationRunner prints, so the
 * two can be compared line for line.
 *
 * This mirrors Code.org's own BaseTestRunner: select each class, build a
 * LauncherDiscoveryRequest, execute it, and record what the listener reports.
 * BaseTestRunner does nothing else that bears on pass/fail — the rest of it is
 * message plumbing (StatusMessage, OutputAdapter) whose only effect is on what
 * the browser is told, not on which tests pass. Compiling it directly would mean
 * shimming half of javabuilder's `lib` module (and the AWS SDK it declares) for
 * no change to a single verdict.
 */
public class RefRunner {
  public static void main(String[] args) throws Exception {
    // JUnit writes progress to stdout; keep it out of the comparison.
    PrintStream out = System.out;
    System.setOut(System.err);

    List<ClassSelector> selectors = new ArrayList<>();
    for (String name : args) {
      selectors.add(DiscoverySelectors.selectClass(Class.forName(name)));
    }
    LauncherDiscoveryRequest request =
        LauncherDiscoveryRequestBuilder.request().selectors(selectors).build();
    Launcher launcher = LauncherFactory.create();
    launcher.registerTestExecutionListeners(
        new TestExecutionListener() {
          @Override
          public void executionFinished(TestIdentifier id, TestExecutionResult result) {
            if (!id.isTest()) {
              return;
            }
            boolean passed = result.getStatus() == TestExecutionResult.Status.SUCCESSFUL;
            String message = result.getThrowable().map(Throwable::getMessage).orElse("");
            if (message == null) {
              message = "";
            }
            out.println(
                "__VTEST\t"
                    + (passed ? "PASS" : "FAIL")
                    + "\t"
                    + id.getDisplayName()
                    + "\t"
                    + message.replace('\t', ' ').replace('\n', ' '));
          }
        });
    TestPlan plan = launcher.discover(request);
    if (plan.containsTests()) {
      launcher.execute(plan);
    }
    out.flush();
  }
}
