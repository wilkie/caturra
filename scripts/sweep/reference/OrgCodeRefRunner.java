import java.io.PrintStream;
import java.lang.reflect.Method;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import org.code.javabuilder.OutputPrintStream;
import org.code.protocol.ClientMessage;
import org.code.protocol.JavabuilderContext;
import org.code.protocol.OutputAdapter;
import org.code.validation.support.NeighborhoodTracker;
import org.code.validation.support.SystemOutTracker;
import org.code.validation.support.ValidationProtocol;
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
 * The reference grading run for levels that use org.code.*: stand up the session
 * the real runtime would, then discover and execute the validator with real
 * JUnit 5, printing one `__VTEST\t<PASS|FAIL>\t<name>\t<message>` line per test.
 *
 * `NeighborhoodTestRunner.run()` and `SystemOutTestRunner.run()` reach through
 * JavabuilderContext for a ValidationProtocol, invoke the student's main, and
 * hand back what the trackers observed — so the harness must build that protocol
 * exactly as javabuilder's CodeExecutionManager does: a NeighborhoodTracker and
 * a SystemOutTracker fed by an OutputAdapter, and System.out redirected through
 * the real OutputPrintStream so a student's println becomes a SYSTEM_OUT message
 * the tracker can see.
 *
 * Usage: OrgCodeRefRunner <MainClass|-> <userClass,userClass,...|-> <TestClass...>
 */
public class OrgCodeRefRunner {
  public static void main(String[] args) throws Exception {
    PrintStream real = System.out;

    String mainClassName = args[0];
    List<String> tests = Arrays.asList(args).subList(2, args.length);

    Method mainMethod = null;
    if (!mainClassName.equals("-")) {
      Class<?> mainClass = Class.forName(mainClassName);
      mainMethod = mainClass.getMethod("main", String[].class);
    }
    // ValidationHelper.getClassNames() hands the validator the student's
    // classes, so it must be all of them, not just the one declaring main.
    List<String> userClassNames = new ArrayList<>();
    if (!args[1].equals("-")) {
      for (String name : args[1].split(",")) {
        if (!name.isEmpty()) {
          userClassNames.add(name);
        }
      }
    }

    NeighborhoodTracker neighborhood = new NeighborhoodTracker();
    SystemOutTracker systemOut = new SystemOutTracker();
    ValidationProtocol protocol =
        new ValidationProtocol(mainMethod, neighborhood, systemOut, userClassNames);
    JavabuilderContext.getInstance().register(ValidationProtocol.class, protocol);

    // Every message a Painter fires, and every line the student prints, reaches
    // the trackers — which is the whole of what the validators then assert on.
    OutputAdapter adapter =
        new OutputAdapter() {
          @Override
          public void sendMessage(ClientMessage message) {
            protocol.trackEvent(message);
          }
        };
    JavabuilderContext.getInstance().getGlobalProtocol().setOutputAdapter(adapter);
    System.setOut(new OutputPrintStream(adapter));

    List<ClassSelector> selectors = new ArrayList<>();
    for (String name : tests) {
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
            real.println(
                "__VTEST\t"
                    + (passed ? "PASS" : "FAIL")
                    + "\t"
                    + id.getDisplayName()
                    + "\t"
                    + message.replace('\t', ' ').replace('\r', ' ').replace('\n', ' '));
          }
        });
    TestPlan plan = launcher.discover(request);
    if (plan.containsTests()) {
      launcher.execute(plan);
    }
    real.flush();
  }
}
