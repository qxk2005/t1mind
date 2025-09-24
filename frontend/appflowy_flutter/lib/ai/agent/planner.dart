class AgentPlanStep {
  AgentPlanStep({required this.endpointName, required this.input});
  final String endpointName;
  final String input;
}

class AgentPlan {
  AgentPlan({required this.steps});
  final List<AgentPlanStep> steps;
}

class AgentPlanner {
  AgentPlan planOnce({
    required String userMessage,
    required List<String> selectedEndpointNames,
    required List<String> allowedEndpointNames,
  }) {
    // Minimal planning: pick the first selected endpoint that is allowed.
    final target = selectedEndpointNames
        .firstWhere((e) => allowedEndpointNames.contains(e), orElse: () {
      // If none allowed matches, fall back to first selected or a placeholder.
      return selectedEndpointNames.isNotEmpty
          ? selectedEndpointNames.first
          : 'mock_tool';
    });
    return AgentPlan(
      steps: [AgentPlanStep(endpointName: target, input: userMessage)],
    );
  }
}


