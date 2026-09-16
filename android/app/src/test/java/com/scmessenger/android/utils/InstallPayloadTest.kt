package com.scmessenger.android.utils

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class InstallPayloadTest {

    @Test
    fun `round-trip without sha preserves apk url and version`() {
        val uri = buildInstallPayloadUri(
            apkUrl = "http://192.168.1.10:41235/scmessenger.apk",
            versionName = "0.4.0",
            versionCode = 15,
            sha256Hex = null
        )

        val parsed = parseInstallPayloadUri(uri)

        assertTrue(parsed != null)
        assertEquals("http://192.168.1.10:41235/scmessenger.apk", parsed!!.apkUrl)
        assertEquals("0.4.0", parsed.versionName)
        assertEquals(15, parsed.versionCode)
        assertNull(parsed.sha256Hex)
    }

    @Test
    fun `round-trip with sha preserves hash`() {
        val sha = "a".repeat(64)
        val uri = buildInstallPayloadUri(
            apkUrl = "http://192.168.1.10:41235/scmessenger.apk",
            versionName = "0.4.0",
            versionCode = 15,
            sha256Hex = sha
        )

        assertTrue(uri.contains("&sha256=$sha"))
        val parsed = parseInstallPayloadUri(uri)

        assertTrue(parsed != null)
        assertEquals(sha, parsed!!.sha256Hex)
    }

    @Test
    fun `build omits sha param when null`() {
        val uri = buildInstallPayloadUri(
            apkUrl = "http://192.168.1.10:8080/scmessenger.apk",
            versionName = "0.4.0",
            versionCode = 15
        )

        assertFalse(uri.contains("sha256"))
        assertNull(parseInstallPayloadUri(uri)!!.sha256Hex)
    }

    @Test
    fun `parse returns null sha when sha absent`() {
        val uri = "scmessenger://install?apk=http%3A%2F%2F192.168.1.10%3A8080%2Fscmessenger.apk&v=0.4.0&vc=15"

        val parsed = parseInstallPayloadUri(uri)

        assertTrue(parsed != null)
        assertNull(parsed!!.sha256Hex)
    }

    @Test
    fun `malformed inputs return null`() {
        val malformed = listOf(
            "",
            "   ",
            "not-a-uri",
            "http://192.168.1.10:8080/scmessenger.apk",
            "scmessenger://invite?public_key=aa",
            "scmessenger://install",
            "scmessenger://install?",
            "scmessenger://install?apk=&v=0.4.0&vc=15",
            "scmessenger://install?v=0.4.0&vc=15",
            "scmessenger://install?apk=http%3A%2F%2F192.168.1.10%2Fa.apk&vc=15",
            "scmessenger://install?apk=http%3A%2F%2F192.168.1.10%2Fa.apk&v=0.4.0",
            "scmessenger://install?apk=http%3A%2F%2F192.168.1.10%2Fa.apk&v=0.4.0&vc=0",
            "scmessenger://install?apk=http%3A%2F%2F192.168.1.10%2Fa.apk&v=0.4.0&vc=abc",
            "scmessenger://install?apk=http%3A%2F%2F192.168.1.10%2Fa.apk&v=0.4.0&vc=-1",
            "scmessenger://install?apk=http%3A%2F%2F192.168.1.10%2Fa.apk&v=0.4.0&vc=15&sha256=xyz",
            "scmessenger://install?apk=http%3A%2F%2F192.168.1.10%2Fa.apk&v=0.4.0&vc=15&sha256=",
            "scmessenger://install?apk=ftp%3A%2F%2Fexample.com%2Fa.apk&v=0.4.0&vc=15"
        )

        for (input in malformed) {
            assertNull("expected null for: $input", parseInstallPayloadUri(input))
        }
    }

    @Test
    fun `parse never throws on garbage percent encoding`() {
        val garbage = listOf(
            "scmessenger://install?apk=%ZZ&v=0.4.0&vc=15",
            "scmessenger://install?apk=%&v=%&vc=%",
            "scmessenger://install?noequals",
            "scmessenger://install?=novalue&v=0.4.0&vc=15"
        )

        for (input in garbage) {
            assertNull("expected null, never throw, for: $input", parseInstallPayloadUri(input))
        }
    }

    @Test
    fun `apk url with port round-trips through encoding`() {
        val apkUrl = "http://10.0.0.5:32768/scmessenger.apk"
        val uri = buildInstallPayloadUri(
            apkUrl = apkUrl,
            versionName = "0.4.0",
            versionCode = 15
        )

        assertTrue(uri.startsWith("scmessenger://install?apk="))
        assertEquals(apkUrl, parseInstallPayloadUri(uri)!!.apkUrl)
    }
}
