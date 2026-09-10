package com.scmessenger.android.arch

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import java.io.File

/**
 * Build-killer guard for the ANR defect class (ticket:
 * HANDOFF/todo/ANDROID_FFI_IN_COMPOSITION_BUILD_KILLER_2026-09-10.md,
 * jointly proposed with the CEO/CTO seats on 2026-09-10).
 *
 * Three consecutive recurrences (PlatformBridge ANR 2026-09-09; diagnostics
 * share crash + Settings FFI-in-composition 2026-09-10) were all the same
 * shape: blocking work reachable from a @Composable function. This test
 * fails the build on the SHAPE, not on individual call sites, by scanning
 * the Kotlin sources of the app:
 *
 *   1. a @Composable function body references the MeshRepository facade
 *      (any `meshRepository.` usage) - ViewModels are the only callers;
 *   2. a @Composable function body performs file I/O (`File(`,
 *      writeText/readText/readLines/... families);
 *   3. a @Composable function body uses `runBlocking`.
 *
 * It runs as an ordinary JVM unit test, so every `testDebugUnitTest` pass
 * (local and CI) enforces it - ticket option 2/3 hybrid, no detekt plugin
 * wiring needed.
 *
 * v1 limitations, accepted and documented:
 *   - line-based scanner: `@Composable` on a val-lambda property (not a
 *     fun declaration) is not tracked;
 *   - the allowlist is (path-fragment, line-fragment) pairs and is capped
 *     at 10 entries by [allowlistStaysSmall] per the ticket's acceptance
 *     criteria ("existing code passes without an allowlist larger than
 *     ~10 items").
 */
class CompositionBlockingCallGuardTest {

    /** Real-tree gate: current sources must be clean or explicitly allowlisted. */
    @Test
    fun `no blocking calls reachable from composable functions in app sources`() {
        val root = findMainSourceRoot()
            ?: error("Could not locate app main source root from user.dir=${System.getProperty("user.dir")}")

        val violations = CompositionBlockingCallScanner.scanTree(
            root,
            ALLOWLIST
        )

        assertTrue(
            "Blocking-call guard tripped - fix the code, not the guard:\n" +
                violations.joinToString("\n") { v ->
                    "  ${v.file}:${v.line}: ${v.reason} :: ${v.snippet.trim()}"
                },
            violations.isEmpty()
        )
    }

    /**
     * Trip-wire: prove the scanner flags the exact reintroduction shapes the
     * guard exists for, and does NOT flag the same calls outside composables.
     * String fixtures only - no temp files (repo rule: tmp/ only).
     */
    @Test
    fun `scanner trips on blocking shapes inside composables and stays quiet outside`() {
        val fixture = """
            package test

            @Composable
            fun BadScreen() {
                val count = meshRepository.getContactCount()
                val f = File("diag.txt")
                data.writeText("payload")
            }

            fun GoodHelper() {
                val count = meshRepository.getContactCount()
                val f = File("diag.txt")
                data.writeText("payload")
            }

            @Composable
            fun AlsoBadScreen() {
                runBlocking { }
            }

            @Composable
            fun CleanScreen() {
                val s = viewModel.infoCounts.value
            }
        """.trimIndent()

        val violations = CompositionBlockingCallScanner.scanSource(fixture)

        val flaggedLines = violations.map { it.line }.sorted()
        // BadScreen body lines 5-7 (repo call, File ctor, writeText) + line 18
        // (runBlocking in AlsoBadScreen). GoodHelper's identical lines 11-13
        // must NOT be flagged (not composable); CleanScreen stays quiet too.
        assertEquals(listOf(5, 6, 7, 18), flaggedLines)
    }

    /** Ticket acceptance criterion: the allowlist must stay small. */
    @Test
    fun allowlistStaysSmall() {
        assertTrue(
            "Allowlist grew to ${ALLOWLIST.size} entries (>10). " +
                "The guard is being neutered - fix the code instead.",
            ALLOWLIST.size <= 10
        )
    }

    /**
     * Deliberately empty as of 2026-09-10 (the recurrence-control pass removed
     * every offending call site). Entries are (path-fragment, line-fragment)
     * pairs; a violation is suppressed only when BOTH fragments match.
     */
    private val ALLOWLIST: List<Pair<String, String>> = emptyList()

    private fun findMainSourceRoot(): File? {
        val candidates = listOfNotNull(
            File(System.getProperty("user.dir"), "src/main/java"),
            // IDE test runs may use the repo root as user.dir.
            File(System.getProperty("user.dir"), "android/app/src/main/java")
        )
        return candidates.firstOrNull { it.isDirectory }
    }
}

internal object CompositionBlockingCallScanner {

    data class Violation(
        val file: String,
        val line: Int,
        val reason: String,
        val snippet: String
    )

    // Violation shapes inside @Composable bodies.
    private val REPO_FACADE = Regex("""\bmeshRepository\s*\.""")
    private val FILE_CTOR = Regex("""\bFile\s*\(""")
    private val IO_METHODS = Regex(
       """\.(writeText|readText|readLines|writeBytes|readBytes|inputStream|outputStream|bufferedReader|bufferedWriter)\s*\("""
    )
    private val RUN_BLOCKING = Regex("""\brunBlocking\b""")

    private val FUN_DECL = Regex("""\bfun\s+\w+\s*\(""")
    private val STRING_LIT = Regex("\"\"\"[\\s\\S]*?\"\"\"|\"(?:[^\"\\\\\\r\\n]|\\\\.)*\"")

    fun scanTree(root: File, allowlist: List<Pair<String, String>> = emptyList()): List<Violation> {
        return root.walkTopDown()
            .filter { it.isFile && it.extension == "kt" }
            .flatMap { file ->
                val relPath = file.relativeTo(root).invariantSeparatorsPath
                scanSource(file.readText(), relPath, allowlist)
            }
            .toList()
    }

    fun scanSource(
        source: String,
        fileName: String = "fixture.kt",
        allowlist: List<Pair<String, String>> = emptyList()
    ): List<Violation> {
        val violations = mutableListOf<Violation>()

        // Local class: must be declared before first reference.
        data class PendingFunction(val startDepth: Int, val composable: Boolean)

        var depth = 0
        var pendingFn: PendingFunction? = null
        var inBlockComment = false
        val declBuffer = mutableListOf<String>()

        source.lines().forEachIndexed { index, rawLine ->
            val lineNo = index + 1

            // Strip block comments (stateful across lines) and line comments.
            var work = rawLine
            if (inBlockComment) {
                val end = work.indexOf("*/")
                if (end < 0) return@forEachIndexed
                work = work.substring(end + 2)
                inBlockComment = false
            }
            while (true) {
                val start = work.indexOf("/*")
                if (start < 0) break
                val end = work.indexOf("*/", start + 2)
                work = if (end < 0) {
                    inBlockComment = true
                    work.substring(0, start)
                } else {
                    work.substring(0, start) + work.substring(end + 2)
                }
                if (inBlockComment) break
            }
            if (inBlockComment) return@forEachIndexed

            val noStrings = STRING_LIT.replace(work, "\"\"")
            val noComments = noStrings.substringBefore("//")

            val activeFn = pendingFn
            if (activeFn != null && depth > activeFn.startDepth && activeFn.composable) {
                check(noComments, lineNo, fileName, rawLine, allowlist, violations)
            }

            if (FUN_DECL.containsMatchIn(noComments)) {
                val isComposable = declBuffer.any { it.contains("@Composable") } ||
                    noComments.contains("@Composable")
                val opens = noComments.count { it == '{' }
                val closes = noComments.count { it == '}' }
                if (opens > closes) {
                    pendingFn = PendingFunction(startDepth = depth, composable = isComposable)
                } else if (isComposable) {
                    // Single-line or expression-bodied composable: check its own line.
                    check(noComments, lineNo, fileName, rawLine, allowlist, violations)
                }
                declBuffer.clear()
            } else if (noComments.contains('}')) {
                declBuffer.clear()
            } else if (noComments.isNotBlank()) {
                declBuffer.add(noComments)
            }

            depth += noComments.count { it == '{' } - noComments.count { it == '}' }
            val closingFn = pendingFn
            if (closingFn != null && depth <= closingFn.startDepth) {
                pendingFn = null
            }
        }

        return violations
    }

    private fun check(
        strippedLine: String,
        lineNo: Int,
        fileName: String,
        rawLine: String,
        allowlist: List<Pair<String, String>>,
        violations: MutableList<Violation>
    ) {
        val reason = when {
            REPO_FACADE.containsMatchIn(strippedLine) ->
                "MeshRepository facade referenced in @Composable body"
            FILE_CTOR.containsMatchIn(strippedLine) ->
                "File I/O (constructor) in @Composable body"
            IO_METHODS.containsMatchIn(strippedLine) ->
                "Blocking file I/O method in @Composable body"
            RUN_BLOCKING.containsMatchIn(strippedLine) ->
                "runBlocking in @Composable body"
            else -> null
        } ?: return

        val allowlisted = allowlist.any { (pathFrag, lineFrag) ->
            fileName.contains(pathFrag) && rawLine.contains(lineFrag)
        }
        if (!allowlisted) {
            violations.add(Violation(fileName, lineNo, reason, rawLine))
        }
    }
}
