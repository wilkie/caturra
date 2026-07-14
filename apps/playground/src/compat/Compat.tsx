import { openJvmSession, type JvmSessionApi } from '@caturra/core';
import { useEffect, useMemo, useRef, useState } from 'react';

import features from './features.json';

/**
 * The Java compatibility page: what caturra implements, and PROOF, run in front of
 * you.
 *
 * Every claim here is a program. A "supported" feature is one whose output matched
 * a real JDK 11's output byte for byte when the manifest was recorded
 * (`scripts/compat/record.py`), and `tests/compat_manifest.rs` re-checks that
 * against a real `javac`/`java` on every CI run. A "gap" is real Java 11 that javac
 * ACCEPTS and caturra declines — with the reason it gives you.
 *
 * Pressing Run compiles and runs the program in this browser tab, in the same WASM
 * engine the playground uses, and compares what comes back with what the JDK
 * printed. So the page cannot claim support it does not have: if the engine
 * regresses, the card in front of you turns red.
 */

type Status = 'supported' | 'unsupported' | 'beyond-11';

interface Feature {
  id: string;
  category: string;
  title: string;
  summary: string;
  main: string;
  source: string;
  status: Status;
  expected?: string;
  reason?: string;
  javac?: string;
}

/** What the browser found when it actually ran the thing. */
interface Outcome {
  state: 'running' | 'pass' | 'fail';
  output: string;
  detail?: string;
}

const ALL = features as Feature[];

const STATUS_LABEL: Record<Status, string> = {
  supported: 'Implemented',
  unsupported: 'Not implemented',
  'beyond-11': 'Newer than Java 11',
};

/** What the button proves, which differs by claim. */
const VERB: Record<Status, string> = {
  supported: 'Run it',
  unsupported: 'Show the error',
  'beyond-11': 'Show the error',
};

function Badge({ status }: { status: Status }) {
  return <span className={`badge badge-${status}`}>{STATUS_LABEL[status]}</span>;
}

function Card({
  feature,
  outcome,
  onRun,
  busy,
}: {
  feature: Feature;
  outcome: Outcome | undefined;
  onRun: () => void;
  busy: boolean;
}) {
  const [open, setOpen] = useState(false);
  return (
    <article className={`card card-${feature.status}`} data-feature={feature.id}>
      <header>
        <h3>{feature.title}</h3>
        <Badge status={feature.status} />
      </header>
      <p className="summary">{feature.summary}</p>

      {feature.status !== 'supported' && feature.reason && (
        <p className="reason">
          <strong>caturra says:</strong> {feature.reason}
        </p>
      )}

      <div className="actions">
        <button type="button" onClick={onRun} disabled={busy} data-run={feature.id}>
          {busy ? 'running…' : VERB[feature.status]}
        </button>
        <button
          type="button"
          className="link"
          onClick={() => {
            setOpen(!open);
          }}
        >
          {open ? 'hide the program' : 'read the program'}
        </button>
        {outcome && (
          <span className={`verdict verdict-${outcome.state}`} data-verdict={feature.id}>
            {outcome.state === 'running' && 'running…'}
            {outcome.state === 'pass' &&
              (feature.status === 'supported'
                ? 'matches the JDK'
                : 'declined, exactly as documented')}
            {outcome.state === 'fail' && 'DOES NOT MATCH'}
          </span>
        )}
      </div>

      {open && <pre className="source">{feature.source}</pre>}

      {outcome && outcome.state !== 'running' && (
        <div className="io">
          <div>
            <h4>{feature.status === 'supported' ? 'in your browser, just now' : 'what you get'}</h4>
            <pre className="out">{outcome.output || '(no output)'}</pre>
          </div>
          {feature.status === 'supported' && (
            <div>
              <h4>a real JDK 11</h4>
              <pre className="out">{feature.expected}</pre>
            </div>
          )}
        </div>
      )}
      {outcome?.detail && <p className="detail">{outcome.detail}</p>}
    </article>
  );
}

export function Compat() {
  const [outcomes, setOutcomes] = useState<Record<string, Outcome>>({});
  const [busy, setBusy] = useState<string | null>(null);
  const [runningAll, setRunningAll] = useState(false);
  const [ready, setReady] = useState(false);
  const sessionRef = useRef<JvmSessionApi | null>(null);

  useEffect(() => {
    let cancelled = false;
    void openJvmSession().then((session) => {
      if (cancelled) {
        session.terminate();
        return;
      }
      sessionRef.current = session;
      setReady(true);
    });
    return () => {
      cancelled = true;
    };
  }, []);

  /**
   * Compile and run one feature's program, here and now, and judge it against what
   * the JDK did. A supported feature has to reproduce the JDK's output exactly; a
   * gap has to be REFUSED, with the reason the page prints — a gap that silently
   * started working would be just as wrong a claim as one that stopped.
   */
  const runOne = async (feature: Feature): Promise<Outcome> => {
    const session = sessionRef.current;
    if (!session) {
      return { state: 'fail', output: '', detail: 'the engine is still loading' };
    }
    const compiled = await session.compile([
      { path: `${feature.main}.java`, text: feature.source },
    ]);

    if (!compiled.success) {
      const message = compiled.diagnostics.find((d) => d.severity === 'error')?.message ?? '';
      const asDocumented = message === feature.reason;
      return {
        state: feature.status === 'supported' ? 'fail' : asDocumented ? 'pass' : 'fail',
        output: message,
        ...(feature.status !== 'supported' && !asDocumented
          ? { detail: `the page says the reason is: ${feature.reason ?? '(none)'}` }
          : {}),
      };
    }

    if (feature.status !== 'supported') {
      return {
        state: 'fail',
        output: '(it compiled)',
        detail: 'the page calls this unsupported, but the engine just accepted it',
      };
    }

    let stdout = '';
    const result = await session.run(feature.main, {
      onStdout: (chunk) => {
        stdout += chunk;
      },
    });
    if (result.status !== 'completed') {
      return { state: 'fail', output: stdout, detail: result.error ?? result.status };
    }
    return {
      state: stdout === feature.expected ? 'pass' : 'fail',
      output: stdout,
    };
  };

  const handleRun = async (feature: Feature) => {
    setBusy(feature.id);
    setOutcomes((prev) => ({ ...prev, [feature.id]: { state: 'running', output: '' } }));
    const outcome = await runOne(feature);
    setOutcomes((prev) => ({ ...prev, [feature.id]: outcome }));
    setBusy(null);
  };

  const handleRunAll = async () => {
    setRunningAll(true);
    for (const feature of ALL) {
      setBusy(feature.id);
      setOutcomes((prev) => ({ ...prev, [feature.id]: { state: 'running', output: '' } }));
      // Sequential on purpose: one engine, and the point is to watch it happen.
      const outcome = await runOne(feature);
      setOutcomes((prev) => ({ ...prev, [feature.id]: outcome }));
    }
    setBusy(null);
    setRunningAll(false);
  };

  const categories = useMemo(() => {
    const grouped = new Map<string, Feature[]>();
    for (const feature of ALL) {
      const list = grouped.get(feature.category) ?? [];
      list.push(feature);
      grouped.set(feature.category, list);
    }
    return [...grouped];
  }, []);

  const counts = useMemo(() => {
    const tally = { supported: 0, unsupported: 0, 'beyond-11': 0 };
    for (const feature of ALL) {
      tally[feature.status] += 1;
    }
    return tally;
  }, []);

  const checked = Object.values(outcomes).filter((o) => o.state !== 'running');
  const failed = checked.filter((o) => o.state === 'fail').length;

  return (
    <main className="compat">
      <header className="masthead">
        <h1>Java compatibility</h1>
        <p className="lede">
          caturra is a Java compiler and JVM written from scratch in Rust, running here as
          WebAssembly. It targets <strong>Java 11</strong>. Everything below is a program, not a
          claim: press <em>Run</em> and it is compiled and executed in this tab, and its output is
          compared with what a real JDK 11 printed for the same program.
        </p>
        <ul className="tally">
          <li>
            <strong>{counts.supported}</strong> implemented
          </li>
          <li>
            <strong>{counts.unsupported}</strong> not implemented
          </li>
          <li>
            <strong>{counts['beyond-11']}</strong> newer than Java 11
          </li>
        </ul>
        <div className="actions">
          <button
            type="button"
            onClick={() => void handleRunAll()}
            disabled={!ready || runningAll}
            data-run-all
          >
            {!ready ? 'loading the engine…' : runningAll ? 'running…' : 'Run every one of them'}
          </button>
          {checked.length > 0 && !runningAll && (
            <span className={failed === 0 ? 'verdict verdict-pass' : 'verdict verdict-fail'}>
              {failed === 0
                ? `${String(checked.length)} verified in your browser`
                : `${String(failed)} of ${String(checked.length)} DID NOT MATCH`}
            </span>
          )}
        </div>
        <p className="fineprint">
          A gap is honest: javac accepts the program and caturra declines it, telling you why.
          &ldquo;Newer than Java 11&rdquo; means javac 11 rejects it too — accepting it would be a
          program that runs here and fails on the JDK the course targets.
        </p>
      </header>

      {categories.map(([category, list]) => (
        <section key={category}>
          <h2>{category}</h2>
          <div className="cards">
            {list.map((feature) => (
              <Card
                key={feature.id}
                feature={feature}
                outcome={outcomes[feature.id]}
                busy={busy === feature.id}
                onRun={() => void handleRun(feature)}
              />
            ))}
          </div>
        </section>
      ))}
    </main>
  );
}
