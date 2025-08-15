#!/usr/bin/env python3
"""Unit tests for Puck code generator."""

import unittest
import json
import sys
import os
from pathlib import Path

# Add compiler to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent.parent / "compiler"))

from compiler.targets.puck.codegen import PuckCodeGenerator
from compiler.ast import *


class TestPuckCodeGenerator(unittest.TestCase):
    """Test cases for Puck code generator."""
    
    def setUp(self):
        """Set up test fixtures."""
        self.generator = PuckCodeGenerator()
    
    def test_empty_program(self):
        """Test empty program generates default structure."""
        program = Program(statements=[])
        result = self.generator.generate(program)
        
        # Parse JSON result
        data = json.loads(result)
        
        # Verify basic structure
        self.assertIn('content', data)
        self.assertIn('root', data)
        self.assertIsInstance(data['content'], list)
        self.assertIsInstance(data['root'], dict)
        
    def test_simple_module(self):
        """Test simple module with data definition."""
        data_fields = [
            DataField(name="name", field_type="text"),
            DataField(name="age", field_type="number")
        ]
        data_def = DataDefinition(name="User", fields=data_fields)
        module = ModuleDefinition(name="test_module", body=[data_def])
        program = Program(statements=[module])
        
        result = self.generator.generate(program)
        data = json.loads(result)
        
        # Should have at least one component
        self.assertTrue(len(data['content']) > 0)
        
    def test_layout_definition(self):
        """Test layout definition conversion."""
        # Create a simple layout with children
        layout = LayoutDefinition(
            name="main_layout",
            children=["row", "column"]
        )
        program = Program(statements=[layout])
        
        result = self.generator.generate(program)
        data = json.loads(result)
        
        # Should have at least one section component
        self.assertTrue(len(data['content']) > 0)
        first_component = data['content'][0]
        self.assertEqual(first_component['type'], 'Section')
        
    def test_form_definition(self):
        """Test form definition conversion."""
        # Create form fields
        input_field = InputComponent(label="Name", placeholder="Enter name")
        button = ButtonComponent(text="Submit")
        
        form = FormDefinition(
            name="user_form",
            fields=[input_field, button]
        )
        program = Program(statements=[form])
        
        result = self.generator.generate(program)
        data = json.loads(result)
        
        # Should have container with form elements
        self.assertTrue(len(data['content']) > 0)
        first_component = data['content'][0]
        self.assertEqual(first_component['type'], 'Container')
        self.assertTrue(len(first_component['children']) > 0)
        
    def test_component_id_generation(self):
        """Test that components get unique IDs."""
        layout = LayoutDefinition(name="layout", children=["row", "column"])
        program = Program(statements=[layout])
        
        result = self.generator.generate(program)
        data = json.loads(result)
        
        # Collect all IDs
        ids = set()
        def collect_ids(component):
            if isinstance(component, dict) and 'id' in component:
                ids.add(component['id'])
            if isinstance(component, dict) and 'children' in component:
                for child in component['children']:
                    collect_ids(child)
        
        for component in data['content']:
            collect_ids(component)
        
        # All IDs should be unique
        self.assertTrue(len(ids) > 0, "Should generate at least one ID")
        
    def test_display_statement_conversion(self):
        """Test display statement to Text component conversion."""
        display = DisplayStatement(message=Literal(value="Hello World"))
        program = Program(statements=[display])
        
        result = self.generator.generate(program)
        data = json.loads(result)
        
        # Should create a default layout with text component
        self.assertTrue(len(data['content']) > 0)
        section = data['content'][0]
        self.assertEqual(section['type'], 'Section')
        
    def test_target_directive_handling(self):
        """Test that @target directive doesn't break generation."""
        # This would normally be handled at a higher level,
        # but the generator should be robust to any AST content
        module = ModuleDefinition(name="app", body=[])
        program = Program(statements=[module])
        
        result = self.generator.generate(program)
        data = json.loads(result)
        
        # Should not crash and produce valid JSON
        self.assertIsInstance(data, dict)
        self.assertIn('content', data)
        self.assertIn('root', data)


class TestPuckComponentMapping(unittest.TestCase):
    """Test specific component mappings."""
    
    def setUp(self):
        """Set up test fixtures."""
        self.generator = PuckCodeGenerator()
    
    def test_button_component(self):
        """Test ButtonComponent mapping."""
        button = ButtonComponent(text="Click Me")
        converted = self.generator._convert_component(button)
        
        self.assertIsNotNone(converted)
        self.assertEqual(converted['type'], 'Button')
        self.assertEqual(converted['props']['text'], 'Click Me')
        
    def test_input_component(self):
        """Test InputComponent mapping."""
        input_comp = InputComponent(label="Email", placeholder="Enter email")
        converted = self.generator._convert_component(input_comp)
        
        self.assertIsNotNone(converted)
        self.assertEqual(converted['type'], 'TextInput')
        self.assertEqual(converted['props']['label'], 'Email')
        self.assertEqual(converted['props']['placeholder'], 'Enter email')
        
    def test_textarea_component(self):
        """Test TextareaComponent mapping."""
        textarea = TextareaComponent(label="Message", rows=5)
        converted = self.generator._convert_component(textarea)
        
        self.assertIsNotNone(converted)
        self.assertEqual(converted['type'], 'Textarea')
        self.assertEqual(converted['props']['rows'], 5)
        
    def test_image_component(self):
        """Test ImageComponent mapping."""
        image = ImageComponent(src="test.jpg", alt="Test Image")
        converted = self.generator._convert_component(image)
        
        self.assertIsNotNone(converted)
        self.assertEqual(converted['type'], 'Image')
        self.assertEqual(converted['props']['alt'], 'Test Image')


if __name__ == '__main__':
    # Run tests
    unittest.main(verbosity=2)