#!/usr/bin/env python3
"""
Unit tests for NVIDIA NIM and metered-backup Cerebras adapters.

Covers:
- key-file mapping (nvidia, cerebras)
- endpoint and default model mappings
- Cerebras max_completion_tokens conversion vs NVIDIA max_tokens
- reasoning_content fallback in response extraction
- disabled-registry skip (generic enabled: false)
- NVIDIA route selection in lake_route.py
"""
import json
import os
import unittest
from unittest.mock import patch, mock_open

import scripts.delegate_task as dt
import scripts.lake_route as lr


class TestFreeLaneAdapters(unittest.TestCase):

    def test_key_file_mapping(self):
        # 1. Environment variable resolution
        with patch.dict(os.environ, {"NVIDIA_API_KEY": "nv-test-key", "CEREBRAS_API_KEY": "cb-test-key"}, clear=True):
            self.assertEqual(dt.get_api_key("nvidia"), "nv-test-key")
            self.assertEqual(dt.get_api_key("cerebras"), "cb-test-key")

        # 2. Env-file resolution fallback
        with patch.dict(os.environ, {}, clear=True):
            with patch("builtins.open", mock_open(read_data="NVIDIA_API_KEY=nv-file-key\n")):
                self.assertEqual(dt.get_api_key("nvidia"), "nv-file-key")

            with patch("builtins.open", mock_open(read_data="CEREBRAS_API_KEY=cb-file-key\n")):
                self.assertEqual(dt.get_api_key("cerebras"), "cb-file-key")

    def test_endpoint_and_default_mappings(self):
        # Endpoints
        self.assertEqual(dt.PROVIDER_URLS.get("nvidia"), "https://integrate.api.nvidia.com/v1/chat/completions")
        self.assertEqual(dt.PROVIDER_URLS.get("cerebras"), "https://api.cerebras.ai/v1/chat/completions")
        self.assertEqual(dt.PROVIDER_URLS.get("gemini"), "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions")

        # Default models
        self.assertEqual(dt.DEFAULT_MODELS.get("nvidia"), "deepseek-ai/deepseek-v4-flash-0731")
        self.assertEqual(dt.DEFAULT_MODELS.get("cerebras"), "zai-glm-4.7")
        self.assertNotEqual(dt.DEFAULT_MODELS.get("cerebras"), "gpt-oss-120b")
        self.assertEqual(dt.DEFAULT_MODELS.get("gemini"), "gemini-3.7-flash")
        self.assertNotEqual(dt.DEFAULT_MODELS.get("gemini"), "gemini-2.5-flash")
        self.assertNotEqual(dt.DEFAULT_MODELS.get("gemini"), "gemini-3.1-pro-preview")

    def test_cerebras_token_field_conversion(self):
        # Cerebras must use max_completion_tokens
        cb_payload = dt.build_request_payload(
            provider="cerebras",
            resolved_model="zai-glm-4.7",
            max_tokens=4096,
            system_message="sys",
            prompt="user",
        )
        self.assertEqual(cb_payload.get("max_completion_tokens"), 4096)
        self.assertNotIn("max_tokens", cb_payload)

        # NVIDIA must use max_tokens
        nv_payload = dt.build_request_payload(
            provider="nvidia",
            resolved_model="deepseek-ai/deepseek-v4-flash-0731",
            max_tokens=8192,
            system_message="sys",
            prompt="user",
        )
        self.assertEqual(nv_payload.get("max_tokens"), 8192)
        self.assertNotIn("max_completion_tokens", nv_payload)

        # Existing provider (e.g. groq) preserves max_tokens
        groq_payload = dt.build_request_payload(
            provider="groq",
            resolved_model="llama-3.3-70b-versatile",
            max_tokens=2048,
            system_message="sys",
            prompt="user",
        )
        self.assertEqual(groq_payload.get("max_tokens"), 2048)
        self.assertNotIn("max_completion_tokens", groq_payload)

    def test_reasoning_content_fallback(self):
        # Standard content
        resp_standard = {
            "choices": [{"message": {"content": "output text", "reasoning": None}}]
        }
        self.assertEqual(dt.extract_response_content(resp_standard, "nvidia"), "output text")

        # Empty content, reasoning present
        resp_reasoning = {
            "choices": [{"message": {"content": "", "reasoning": "reasoning text"}}]
        }
        self.assertEqual(dt.extract_response_content(resp_reasoning, "nvidia"), "reasoning text")

        # Empty content, reasoning null, reasoning_content present
        resp_reasoning_content = {
            "choices": [{"message": {"content": None, "reasoning": None, "reasoning_content": "reasoning content text"}}]
        }
        self.assertEqual(dt.extract_response_content(resp_reasoning_content, "nvidia"), "reasoning content text")

        # All empty/null
        resp_empty = {
            "choices": [{"message": {"content": "", "reasoning": "", "reasoning_content": ""}}]
        }
        self.assertEqual(dt.extract_response_content(resp_empty, "cerebras"), "")

    def test_disabled_registry_skip(self):
        mock_registry = {
            "lakes": {
                "cerebras": {
                    "endpoint": "https://api.cerebras.ai/v1/chat/completions",
                    "key_env": ["CEREBRAS_API_KEY"],
                    "enabled": False,
                    "tiers": {"FLASH": ["zai-glm-4.7"]}
                },
                "ollama": {
                    "endpoint": "http://localhost:11434/api/chat",
                    "key_env": [],
                    "enabled": True,
                    "tiers": {"FLASH": ["gemma3:4b"]}
                }
            }
        }
        with patch.dict(os.environ, {"CEREBRAS_API_KEY": "valid-key"}, clear=True):
            with patch("scripts.lake_route._load_json", side_effect=[mock_registry, {}]):
                with patch("scripts.lake_route._load_ledger", return_value={}):
                    with patch("scripts.lake_route._save_rr"):
                        lake, model = lr.route("FLASH")
                        # Cerebras has a valid key but enabled is False, so it must be skipped in favor of ollama
                        self.assertEqual(lake, "ollama")
                        self.assertEqual(model, "gemma3:4b")

    def test_nvidia_route_selection(self):
        mock_registry = {
            "lakes": {
                "nvidia": {
                    "endpoint": "https://integrate.api.nvidia.com/v1/chat/completions",
                    "key_env": ["NVIDIA_API_KEY"],
                    "enabled": True,
                    "tiers": {"FLASH": ["deepseek-ai/deepseek-v4-flash-0731"]}
                },
                "cerebras": {
                    "endpoint": "https://api.cerebras.ai/v1/chat/completions",
                    "key_env": ["CEREBRAS_API_KEY"],
                    "enabled": False,
                    "tiers": {"FLASH": ["zai-glm-4.7"]}
                }
            }
        }
        with patch.dict(os.environ, {"NVIDIA_API_KEY": "test-nv-key", "CEREBRAS_API_KEY": "test-cb-key"}, clear=True):
            with patch("scripts.lake_route._load_json", side_effect=[mock_registry, {}]):
                with patch("scripts.lake_route._load_ledger", return_value={}):
                    with patch("scripts.lake_route._save_rr"):
                        lake, model = lr.route("FLASH")
                        self.assertEqual(lake, "nvidia")
                        self.assertEqual(model, "deepseek-ai/deepseek-v4-flash-0731")


    def test_gemini_key_resolution_environment(self):
        # AISTUDIO_API_KEY resolution
        with patch.dict(os.environ, {"AISTUDIO_API_KEY": "mock-aistudio-env-key"}, clear=True):
            self.assertEqual(dt.get_api_key("gemini"), "mock-aistudio-env-key")

        # GEMINI_API_KEY legacy env fallback
        with patch.dict(os.environ, {"GEMINI_API_KEY": "mock-gemini-env-key"}, clear=True):
            self.assertEqual(dt.get_api_key("gemini"), "mock-gemini-env-key")

        # GOOGLE_API_KEY legacy env fallback
        with patch.dict(os.environ, {"GOOGLE_API_KEY": "mock-google-env-key"}, clear=True):
            self.assertEqual(dt.get_api_key("gemini"), "mock-google-env-key")

        # Precedence: AISTUDIO_API_KEY > GEMINI_API_KEY > GOOGLE_API_KEY
        with patch.dict(os.environ, {
            "AISTUDIO_API_KEY": "mock-priority-key",
            "GEMINI_API_KEY": "mock-secondary-key",
            "GOOGLE_API_KEY": "mock-tertiary-key",
        }, clear=True):
            self.assertEqual(dt.get_api_key("gemini"), "mock-priority-key")

    def test_gemini_key_resolution_file_fallbacks(self):
        # Approved slot fallback (~/.config/scmorc/AIstudio.env)
        def mock_open_side_effect(filepath, *args, **kwargs):
            norm = os.path.normpath(str(filepath))
            if "AIstudio.env" in norm:
                return mock_open(read_data="AISTUDIO_API_KEY=mock-aistudio-file-key\n")()
            raise OSError("File not found")

        with patch.dict(os.environ, {}, clear=True):
            with patch("builtins.open", side_effect=mock_open_side_effect):
                self.assertEqual(dt.get_api_key("gemini"), "mock-aistudio-file-key")

        # Legacy slot fallback (~/.config/scmorc/gemini.env) when AIstudio.env is absent
        def mock_open_legacy_side_effect(filepath, *args, **kwargs):
            norm = os.path.normpath(str(filepath))
            if "AIstudio.env" in norm:
                raise OSError("File not found")
            if "gemini.env" in norm:
                return mock_open(read_data="GEMINI_API_KEY=mock-gemini-file-key\n")()
            raise OSError("File not found")

        with patch.dict(os.environ, {}, clear=True):
            with patch("builtins.open", side_effect=mock_open_legacy_side_effect):
                self.assertEqual(dt.get_api_key("gemini"), "mock-gemini-file-key")

        # Precedence: AIstudio.env > gemini.env
        def mock_open_both_side_effect(filepath, *args, **kwargs):
            norm = os.path.normpath(str(filepath))
            if "AIstudio.env" in norm:
                return mock_open(read_data="AISTUDIO_API_KEY=mock-slot-priority-key\n")()
            if "gemini.env" in norm:
                return mock_open(read_data="GEMINI_API_KEY=mock-slot-secondary-key\n")()
            raise OSError("File not found")

        with patch.dict(os.environ, {}, clear=True):
            with patch("builtins.open", side_effect=mock_open_both_side_effect):
                self.assertEqual(dt.get_api_key("gemini"), "mock-slot-priority-key")

    def test_gemini_key_resolution_fail_closed(self):
        # Absent environment and missing files
        with patch.dict(os.environ, {}, clear=True):
            with patch("builtins.open", side_effect=OSError("No such file")):
                self.assertIsNone(dt.get_api_key("gemini"))

        # Empty keys in file
        with patch.dict(os.environ, {}, clear=True):
            with patch("builtins.open", mock_open(read_data="AISTUDIO_API_KEY=\nGEMINI_API_KEY=   \n")):
                self.assertIsNone(dt.get_api_key("gemini"))

    def test_generic_key_file_lists(self):
        mock_cfg = {
            "endpoint": "https://api.example.com/v1/chat/completions",
            "key_env": ["GENERIC_API_KEY"],
            "key_files": ["~/.config/scmorc/slot1.env", "~/.config/scmorc/slot2.env"],
            "enabled": True,
            "tiers": {"FLASH": ["generic-model"]}
        }
        # First file slot hits
        def mock_open_slot1(filepath, *args, **kwargs):
            norm = os.path.normpath(str(filepath))
            if "slot1.env" in norm:
                return mock_open(read_data="GENERIC_API_KEY=slot1-key\n")()
            raise OSError("File not found")

        with patch.dict(os.environ, {}, clear=True):
            with patch("builtins.open", side_effect=mock_open_slot1):
                self.assertTrue(lr._lake_has_key("generic_custom", mock_cfg))

        # Second file slot hits when first is absent
        def mock_open_slot2(filepath, *args, **kwargs):
            norm = os.path.normpath(str(filepath))
            if "slot1.env" in norm:
                raise OSError("File not found")
            if "slot2.env" in norm:
                return mock_open(read_data="GENERIC_API_KEY=slot2-key\n")()
            raise OSError("File not found")

        with patch.dict(os.environ, {}, clear=True):
            with patch("builtins.open", side_effect=mock_open_slot2):
                self.assertTrue(lr._lake_has_key("generic_custom", mock_cfg))

    def test_gemini_lake_router_key_discovery_and_selection(self):
        gemini_cfg = {
            "endpoint": "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions",
            "key_env": ["AISTUDIO_API_KEY", "GEMINI_API_KEY", "GOOGLE_API_KEY"],
            "key_file": "~/.config/scmorc/AIstudio.env",
            "enabled": True,
            "tiers": {
                "FLASH": ["gemini-3.7-flash"],
                "CODER": ["gemini-3.7-flash"]
            }
        }

        # Router discovers via AISTUDIO_API_KEY in environment
        with patch.dict(os.environ, {"AISTUDIO_API_KEY": "mock-router-key"}, clear=True):
            self.assertTrue(lr._lake_has_key("gemini", gemini_cfg))

        # Router discovers via GEMINI_API_KEY in environment
        with patch.dict(os.environ, {"GEMINI_API_KEY": "mock-router-gemini-key"}, clear=True):
            self.assertTrue(lr._lake_has_key("gemini", gemini_cfg))

        # Router discovers via AIstudio.env file slot
        def mock_open_router(filepath, *args, **kwargs):
            norm = os.path.normpath(str(filepath))
            if "AIstudio.env" in norm:
                return mock_open(read_data="AISTUDIO_API_KEY=mock-slot-key\n")()
            raise OSError("File not found")

        with patch.dict(os.environ, {}, clear=True):
            with patch("builtins.open", side_effect=mock_open_router):
                self.assertTrue(lr._lake_has_key("gemini", gemini_cfg))

        # Router discovers via legacy gemini.env file slot even when AIstudio.env is primary in config
        def mock_open_router_legacy(filepath, *args, **kwargs):
            norm = os.path.normpath(str(filepath))
            if "AIstudio.env" in norm:
                raise OSError("File not found")
            if "gemini.env" in norm:
                return mock_open(read_data="GEMINI_API_KEY=mock-legacy-key\n")()
            raise OSError("File not found")

        with patch.dict(os.environ, {}, clear=True):
            with patch("builtins.open", side_effect=mock_open_router_legacy):
                self.assertTrue(lr._lake_has_key("gemini", gemini_cfg))

        # Router fails closed when absent
        with patch.dict(os.environ, {}, clear=True):
            with patch("builtins.open", side_effect=OSError("File not found")):
                self.assertFalse(lr._lake_has_key("gemini", gemini_cfg))

        # Router fails closed when keys in file are empty
        with patch.dict(os.environ, {}, clear=True):
            with patch("builtins.open", mock_open(read_data="AISTUDIO_API_KEY=\nGEMINI_API_KEY=   \n")):
                self.assertFalse(lr._lake_has_key("gemini", gemini_cfg))

        # Route selection for gemini FLASH
        mock_registry = {
            "lakes": {
                "gemini": gemini_cfg
            }
        }
        with patch.dict(os.environ, {"AISTUDIO_API_KEY": "mock-router-key"}, clear=True):
            with patch("scripts.lake_route._load_json", side_effect=[mock_registry, {}]):
                with patch("scripts.lake_route._load_ledger", return_value={}):
                    with patch("scripts.lake_route._save_rr"):
                        lake, model = lr.route("FLASH")
                        self.assertEqual(lake, "gemini")
                        self.assertEqual(model, "gemini-3.7-flash")
                        self.assertNotEqual(model, "gemini-2.0-flash-lite")
                        self.assertNotEqual(model, "gemini-2.5-flash")

        # Route selection for gemini CODER
        with patch.dict(os.environ, {"AISTUDIO_API_KEY": "mock-router-key"}, clear=True):
            with patch("scripts.lake_route._load_json", side_effect=[mock_registry, {}]):
                with patch("scripts.lake_route._load_ledger", return_value={}):
                    with patch("scripts.lake_route._save_rr"):
                        lake, model = lr.route("CODER")
                        self.assertEqual(lake, "gemini")
                        self.assertEqual(model, "gemini-3.7-flash")
                        self.assertNotEqual(model, "gemini-2.5-flash")

        # Route selection for THINK: Gemini omits THINK tier, so it fails closed / returns None
        with patch.dict(os.environ, {"AISTUDIO_API_KEY": "mock-router-key"}, clear=True):
            with patch("scripts.lake_route._load_json", side_effect=[mock_registry, {}]):
                with patch("scripts.lake_route._load_ledger", return_value={}):
                    with patch("scripts.lake_route._save_rr"):
                        lake, model = lr.route("THINK")
                        self.assertIsNone(lake)
                        self.assertIsNone(model)


if __name__ == "__main__":
    unittest.main()
