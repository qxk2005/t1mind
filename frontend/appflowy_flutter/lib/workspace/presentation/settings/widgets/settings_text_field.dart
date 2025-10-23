import 'package:flowy_infra_ui/style_widget/text.dart';
import 'package:flowy_infra_ui/widget/spacing.dart';
import 'package:flutter/material.dart';

class SettingsTextField extends StatelessWidget {
  const SettingsTextField({
    super.key,
    required this.label,
    required this.value,
    required this.onChanged,
    this.description,
    this.keyboardType,
    this.hintText,
  });

  final String label;
  final String value;
  final ValueChanged<String> onChanged;
  final String? description;
  final TextInputType? keyboardType;
  final String? hintText;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        FlowyText.regular(
          label,
          fontSize: 16,
        ),
        if (description != null) ...[
          const VSpace(4),
          FlowyText.small(
            description!,
            color: Theme.of(context).colorScheme.onSurfaceVariant,
          ),
        ],
        const VSpace(8),
        SizedBox(
          height: 40,
          child: TextField(
            controller: TextEditingController(text: value),
            keyboardType: keyboardType,
            decoration: InputDecoration(
              hintText: hintText,
              border: OutlineInputBorder(
                borderRadius: BorderRadius.circular(8),
              ),
              contentPadding: const EdgeInsets.symmetric(
                horizontal: 12,
                vertical: 8,
              ),
            ),
            onChanged: onChanged,
          ),
        ),
      ],
    );
  }
}
