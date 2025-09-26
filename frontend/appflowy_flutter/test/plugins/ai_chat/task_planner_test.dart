import 'package:appflowy/plugins/ai_chat/application/task_planner_bloc.dart';
import 'package:appflowy/plugins/ai_chat/application/task_planner_entities.dart';
import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:nanoid/nanoid.dart';

void main() {
  group('TaskPlannerBloc', () {
    const testSessionId = 'test-session-123';
    const testUserId = 'test-user-456';
    const testUserQuery = '帮我分析这个文档并生成摘要';
    const testAgentId = 'test-agent-789';
    final testMcpTools = ['file-reader', 'text-analyzer', 'summary-generator'];

    late TaskPlannerBloc bloc;

    setUp(() {
      bloc = TaskPlannerBloc(
        sessionId: testSessionId,
        userId: testUserId,
      );
    });

    tearDown(() {
      bloc.close();
    });

    group('初始状态', () {
      test('应该返回正确的初始状态', () {
        expect(bloc.state.status, TaskPlannerStatus.idle);
        expect(bloc.state.currentTaskPlan, isNull);
        expect(bloc.state.planHistory, isEmpty);
        expect(bloc.state.errorMessage, isNull);
        expect(bloc.state.lastUpdated, isNotNull);
      });

      test('应该正确设置sessionId和userId', () {
        expect(bloc.sessionId, testSessionId);
        expect(bloc.userId, testUserId);
      });
    });

    group('状态扩展方法', () {
      test('hasCurrentPlan应该正确返回', () {
        expect(bloc.state.hasCurrentPlan, isFalse);
      });

      test('canCreateNewPlan应该在idle状态下返回true', () {
        expect(bloc.state.canCreateNewPlan, isTrue);
      });

      test('isProcessing应该在非processing状态下返回false', () {
        expect(bloc.state.isProcessing, isFalse);
      });

      test('needsConfirmation应该在非waitingConfirmation状态下返回false', () {
        expect(bloc.state.needsConfirmation, isFalse);
      });

      test('hasError应该在无错误时返回false', () {
        expect(bloc.state.hasError, isFalse);
      });
    });

    group('创建任务规划', () {
      blocTest<TaskPlannerBloc, TaskPlannerState>(
        '应该成功创建任务规划',
        build: () => TaskPlannerBloc(
          sessionId: testSessionId,
          userId: testUserId,
        ),
        act: (bloc) => bloc.add(
          TaskPlannerEvent.createTaskPlan(
            userQuery: testUserQuery,
            mcpTools: testMcpTools,
            agentId: testAgentId,
          ),
        ),
        wait: const Duration(milliseconds: 600),
        expect: () => [
          // 开始规划状态
          isA<TaskPlannerState>()
              .having((s) => s.status, 'status', TaskPlannerStatus.planning)
              .having((s) => s.errorMessage, 'errorMessage', isNull),
          // 等待确认状态
          isA<TaskPlannerState>()
              .having((s) => s.status, 'status', TaskPlannerStatus.waitingConfirmation)
              .having((s) => s.currentTaskPlan, 'currentTaskPlan', isNotNull)
              .having((s) => s.currentTaskPlan!.userQuery, 'userQuery', testUserQuery)
              .having((s) => s.currentTaskPlan!.requiredMcpTools, 'mcpTools', testMcpTools)
              .having((s) => s.currentTaskPlan!.agentId, 'agentId', testAgentId)
              .having((s) => s.currentTaskPlan!.sessionId, 'sessionId', testSessionId)
              .having((s) => s.currentTaskPlan!.status, 'planStatus', TaskPlanStatus.pendingConfirmation)
              .having((s) => s.currentTaskPlan!.steps, 'steps', isNotEmpty)
              .having((s) => s.planHistory, 'planHistory', hasLength(1))
              .having((s) => s.errorMessage, 'errorMessage', isNull),
        ],
      );

      blocTest<TaskPlannerBloc, TaskPlannerState>(
        '应该为空工具列表生成默认步骤',
        build: () => TaskPlannerBloc(
          sessionId: testSessionId,
          userId: testUserId,
        ),
        act: (bloc) => bloc.add(
          TaskPlannerEvent.createTaskPlan(
            userQuery: testUserQuery,
            mcpTools: [],
          ),
        ),
        wait: const Duration(milliseconds: 600),
        expect: () => [
          isA<TaskPlannerState>()
              .having((s) => s.status, 'status', TaskPlannerStatus.planning),
          isA<TaskPlannerState>()
              .having((s) => s.status, 'status', TaskPlannerStatus.waitingConfirmation)
              .having((s) => s.currentTaskPlan!.steps, 'steps', hasLength(5))
              .having((s) => s.currentTaskPlan!.steps.first.mcpToolId, 'defaultTool', 'ai-assistant'),
        ],
      );

      blocTest<TaskPlannerBloc, TaskPlannerState>(
        '应该在已有操作进行时拒绝新的创建请求',
        build: () => TaskPlannerBloc(
          sessionId: testSessionId,
          userId: testUserId,
        ),
        seed: () => TaskPlannerState(
          status: TaskPlannerStatus.planning,
          lastUpdated: DateTime.now(),
        ),
        act: (bloc) => bloc.add(
          TaskPlannerEvent.createTaskPlan(
            userQuery: testUserQuery,
            mcpTools: testMcpTools,
          ),
        ),
        wait: const Duration(milliseconds: 100),
        expect: () => [],
      );
    });

    group('确认任务规划', () {
      blocTest<TaskPlannerBloc, TaskPlannerState>(
        '应该成功确认任务规划',
        build: () => TaskPlannerBloc(
          sessionId: testSessionId,
          userId: testUserId,
        ),
        act: (bloc) async {
          // 首先创建一个任务规划
          bloc.add(
            TaskPlannerEvent.createTaskPlan(
              userQuery: testUserQuery,
              mcpTools: testMcpTools,
              agentId: testAgentId,
            ),
          );
          
          // 等待规划创建完成
          await Future.delayed(const Duration(milliseconds: 600));
          
          // 获取创建的任务规划ID
          final taskPlanId = bloc.state.currentTaskPlan?.id;
          if (taskPlanId != null) {
            bloc.add(TaskPlannerEvent.confirmTaskPlan(taskPlanId: taskPlanId));
          }
        },
        wait: const Duration(milliseconds: 700),
        expect: () => [
          // 创建规划的状态变化
          isA<TaskPlannerState>()
              .having((s) => s.status, 'status', TaskPlannerStatus.planning),
          isA<TaskPlannerState>()
              .having((s) => s.status, 'status', TaskPlannerStatus.waitingConfirmation),
          // 确认规划的状态变化
          isA<TaskPlannerState>()
              .having((s) => s.status, 'status', TaskPlannerStatus.planReady)
              .having((s) => s.currentTaskPlan!.status, 'planStatus', TaskPlanStatus.confirmed)
              .having((s) => s.currentTaskPlan!.updatedAt, 'updatedAt', isNotNull)
              .having((s) => s.errorMessage, 'errorMessage', isNull),
        ],
      );

      blocTest<TaskPlannerBloc, TaskPlannerState>(
        '应该拒绝确认错误的任务规划ID',
        build: () => TaskPlannerBloc(
          sessionId: testSessionId,
          userId: testUserId,
        ),
        act: (bloc) => bloc.add(
          TaskPlannerEvent.confirmTaskPlan(taskPlanId: 'wrong-id'),
        ),
        expect: () => [],
      );
    });

    group('拒绝任务规划', () {
      const rejectReason = '规划不符合要求';

      blocTest<TaskPlannerBloc, TaskPlannerState>(
        '应该成功拒绝任务规划',
        build: () => TaskPlannerBloc(
          sessionId: testSessionId,
          userId: testUserId,
        ),
        act: (bloc) async {
          // 首先创建一个任务规划
          bloc.add(
            TaskPlannerEvent.createTaskPlan(
              userQuery: testUserQuery,
              mcpTools: testMcpTools,
            ),
          );
          
          // 等待规划创建完成
          await Future.delayed(const Duration(milliseconds: 600));
          
          // 获取创建的任务规划ID并拒绝
          final taskPlanId = bloc.state.currentTaskPlan?.id;
          if (taskPlanId != null) {
            bloc.add(
              TaskPlannerEvent.rejectTaskPlan(
                taskPlanId: taskPlanId,
                reason: rejectReason,
              ),
            );
          }
        },
        wait: const Duration(milliseconds: 700),
        expect: () => [
          // 创建规划的状态变化
          isA<TaskPlannerState>()
              .having((s) => s.status, 'status', TaskPlannerStatus.planning),
          isA<TaskPlannerState>()
              .having((s) => s.status, 'status', TaskPlannerStatus.waitingConfirmation),
          // 拒绝规划的状态变化
          isA<TaskPlannerState>()
              .having((s) => s.status, 'status', TaskPlannerStatus.idle)
              .having((s) => s.currentTaskPlan!.status, 'planStatus', TaskPlanStatus.rejected)
              .having((s) => s.currentTaskPlan!.errorMessage, 'errorMessage', rejectReason)
              .having((s) => s.currentTaskPlan!.updatedAt, 'updatedAt', isNotNull)
              .having((s) => s.errorMessage, 'errorMessage', isNull),
        ],
      );

      blocTest<TaskPlannerBloc, TaskPlannerState>(
        '应该拒绝拒绝错误的任务规划ID',
        build: () => TaskPlannerBloc(
          sessionId: testSessionId,
          userId: testUserId,
        ),
        act: (bloc) => bloc.add(
          TaskPlannerEvent.rejectTaskPlan(
            taskPlanId: 'wrong-id',
            reason: rejectReason,
          ),
        ),
        expect: () => [],
      );
    });

    group('任务步骤管理', () {
      blocTest<TaskPlannerBloc, TaskPlannerState>(
        '应该成功添加任务步骤',
        build: () => TaskPlannerBloc(
          sessionId: testSessionId,
          userId: testUserId,
        ),
        act: (bloc) async {
          // 首先创建并确认一个任务规划
          bloc.add(
            TaskPlannerEvent.createTaskPlan(
              userQuery: testUserQuery,
              mcpTools: testMcpTools,
            ),
          );
          
          await Future.delayed(const Duration(milliseconds: 600));
          
          final taskPlanId = bloc.state.currentTaskPlan?.id;
          if (taskPlanId != null) {
            bloc.add(TaskPlannerEvent.confirmTaskPlan(taskPlanId: taskPlanId));
            await Future.delayed(const Duration(milliseconds: 100));
            
            // 添加新步骤
            final newStep = TaskStep(
              id: nanoid(),
              description: '新步骤',
              mcpToolId: 'new-tool',
              estimatedDurationSeconds: 45,
            );
            bloc.add(
              TaskPlannerEvent.addTaskStep(
                taskPlanId: taskPlanId,
                step: newStep,
              ),
            );
          }
        },
        wait: const Duration(milliseconds: 800),
        skip: 3, // 跳过创建和确认的状态变化
        expect: () => [
          isA<TaskPlannerState>()
              .having((s) => s.currentTaskPlan!.steps.length, 'stepsLength', greaterThan(3)),
        ],
      );
    });

    group('执行控制', () {
      blocTest<TaskPlannerBloc, TaskPlannerState>(
        '应该成功开始执行',
        build: () => TaskPlannerBloc(
          sessionId: testSessionId,
          userId: testUserId,
        ),
        act: (bloc) async {
          // 创建并确认任务规划
          bloc.add(
            TaskPlannerEvent.createTaskPlan(
              userQuery: testUserQuery,
              mcpTools: testMcpTools,
            ),
          );
          
          await Future.delayed(const Duration(milliseconds: 600));
          
          final taskPlanId = bloc.state.currentTaskPlan?.id;
          if (taskPlanId != null) {
            bloc.add(TaskPlannerEvent.confirmTaskPlan(taskPlanId: taskPlanId));
            await Future.delayed(const Duration(milliseconds: 100));
            
            // 开始执行
            bloc.add(TaskPlannerEvent.startExecution(taskPlanId: taskPlanId));
          }
        },
        wait: const Duration(milliseconds: 800),
        skip: 3, // 跳过创建和确认的状态变化
        expect: () => [
          isA<TaskPlannerState>()
              .having((s) => s.currentTaskPlan!.status, 'planStatus', TaskPlanStatus.executing),
        ],
      );

      blocTest<TaskPlannerBloc, TaskPlannerState>(
        '应该拒绝执行错误的任务规划ID',
        build: () => TaskPlannerBloc(
          sessionId: testSessionId,
          userId: testUserId,
        ),
        act: (bloc) => bloc.add(
          TaskPlannerEvent.startExecution(taskPlanId: 'wrong-id'),
        ),
        expect: () => [],
      );
    });

    group('状态管理', () {
      blocTest<TaskPlannerBloc, TaskPlannerState>(
        '应该成功清除当前规划',
        build: () => TaskPlannerBloc(
          sessionId: testSessionId,
          userId: testUserId,
        ),
        act: (bloc) async {
          // 先创建一个规划
          bloc.add(
            TaskPlannerEvent.createTaskPlan(
              userQuery: testUserQuery,
              mcpTools: testMcpTools,
            ),
          );
          
          await Future.delayed(const Duration(milliseconds: 600));
          
          // 清除当前规划
          bloc.add(const TaskPlannerEvent.clearCurrentPlan());
        },
        wait: const Duration(milliseconds: 700),
        skip: 2, // 跳过创建规划的状态变化
        expect: () => [
          isA<TaskPlannerState>()
              .having((s) => s.status, 'status', TaskPlannerStatus.idle)
              .having((s) => s.currentTaskPlan, 'currentTaskPlan', isNull)
              .having((s) => s.errorMessage, 'errorMessage', isNull),
        ],
      );

      blocTest<TaskPlannerBloc, TaskPlannerState>(
        '应该成功加载规划历史',
        build: () => TaskPlannerBloc(
          sessionId: testSessionId,
          userId: testUserId,
        ),
        act: (bloc) => bloc.add(const TaskPlannerEvent.loadPlanHistory()),
        expect: () => [
          isA<TaskPlannerState>()
              .having((s) => s.planHistory, 'planHistory', isA<List<TaskPlan>>()),
        ],
      );

      blocTest<TaskPlannerBloc, TaskPlannerState>(
        '应该成功清除错误',
        build: () => TaskPlannerBloc(
          sessionId: testSessionId,
          userId: testUserId,
        ),
        seed: () => TaskPlannerState(
          status: TaskPlannerStatus.planFailed,
          errorMessage: '测试错误信息',
          lastUpdated: DateTime.now(),
        ),
        act: (bloc) => bloc.add(const TaskPlannerEvent.clearError()),
        expect: () => [
          isA<TaskPlannerState>()
              .having((s) => s.errorMessage, 'errorMessage', isNull),
        ],
      );
    });

    group('错误处理', () {
      blocTest<TaskPlannerBloc, TaskPlannerState>(
        '应该处理重试上次操作',
        build: () => TaskPlannerBloc(
          sessionId: testSessionId,
          userId: testUserId,
        ),
        seed: () => TaskPlannerState(
          status: TaskPlannerStatus.planFailed,
          errorMessage: '测试错误信息',
          lastUpdated: DateTime.now(),
        ),
        act: (bloc) => bloc.add(const TaskPlannerEvent.retryLastOperation()),
        expect: () => [
          isA<TaskPlannerState>()
              .having((s) => s.errorMessage, 'errorMessage', isNull),
        ],
      );
    });

    group('边界情况和异常处理', () {
      blocTest<TaskPlannerBloc, TaskPlannerState>(
        '应该处理空用户查询',
        build: () => TaskPlannerBloc(
          sessionId: testSessionId,
          userId: testUserId,
        ),
        act: (bloc) => bloc.add(
          TaskPlannerEvent.createTaskPlan(
            userQuery: '',
            mcpTools: testMcpTools,
          ),
        ),
        wait: const Duration(milliseconds: 600),
        expect: () => [
          isA<TaskPlannerState>()
              .having((s) => s.status, 'status', TaskPlannerStatus.planning),
          isA<TaskPlannerState>()
              .having((s) => s.status, 'status', TaskPlannerStatus.waitingConfirmation)
              .having((s) => s.currentTaskPlan!.userQuery, 'userQuery', ''),
        ],
      );

      test('应该正确处理bloc关闭', () async {
        final testBloc = TaskPlannerBloc(
          sessionId: testSessionId,
          userId: testUserId,
        );

        // 启动一个长时间运行的操作
        testBloc.add(
          TaskPlannerEvent.createTaskPlan(
            userQuery: testUserQuery,
            mcpTools: testMcpTools,
          ),
        );

        // 立即关闭bloc
        await testBloc.close();

        // 验证bloc已关闭
        expect(testBloc.isClosed, isTrue);
      });
    });

    group('并发操作处理', () {
      blocTest<TaskPlannerBloc, TaskPlannerState>(
        '应该正确处理并发的创建请求',
        build: () => TaskPlannerBloc(
          sessionId: testSessionId,
          userId: testUserId,
        ),
        act: (bloc) {
          // 同时发送多个创建请求
          bloc.add(
            TaskPlannerEvent.createTaskPlan(
              userQuery: '第一个查询',
              mcpTools: ['tool1'],
            ),
          );
          bloc.add(
            TaskPlannerEvent.createTaskPlan(
              userQuery: '第二个查询',
              mcpTools: ['tool2'],
            ),
          );
        },
        wait: const Duration(milliseconds: 600),
        expect: () => [
          // 只应该处理第一个请求
          isA<TaskPlannerState>()
              .having((s) => s.status, 'status', TaskPlannerStatus.planning),
          isA<TaskPlannerState>()
              .having((s) => s.status, 'status', TaskPlannerStatus.waitingConfirmation)
              .having((s) => s.currentTaskPlan!.userQuery, 'userQuery', '第一个查询'),
        ],
      );
    });

    group('状态转换验证', () {
      test('TaskPlanStatus枚举方法应该正确工作', () {
        expect(TaskPlanStatus.confirmed.canExecute, isTrue);
        expect(TaskPlanStatus.pendingConfirmation.canExecute, isFalse);
        expect(TaskPlanStatus.executing.isExecuting, isTrue);
        expect(TaskPlanStatus.confirmed.isExecuting, isFalse);
        expect(TaskPlanStatus.completed.isFinished, isTrue);
        expect(TaskPlanStatus.confirmed.isFinished, isFalse);
      });

      test('TaskPlannerStatus枚举方法应该正确工作', () {
        expect(TaskPlannerStatus.planning.isProcessing, isTrue);
        expect(TaskPlannerStatus.idle.isProcessing, isFalse);
        expect(TaskPlannerStatus.waitingConfirmation.needsUserAction, isTrue);
        expect(TaskPlannerStatus.idle.needsUserAction, isFalse);
      });
    });
  });
}