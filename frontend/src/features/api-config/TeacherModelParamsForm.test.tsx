/**
 * TeacherModelParamsForm 测试
 * Story 1.5 Task 10.3: 范围校验与保存禁用逻辑测试
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { TeacherModelParamsForm } from './TeacherModelParamsForm';
import { useCredentialStore } from '@/stores/useCredentialStore';
import { defaultTeacherSettings, teacherSettingsConstraints } from '@/types/credentials';

describe('TeacherModelParamsForm', () => {
  beforeEach(() => {
    // 重置 Store 状态
    useCredentialStore.setState({
      teacherSettings: { ...defaultTeacherSettings },
      isDirty: false,
    });
  });

  describe('渲染', () => {
    it('应该正确渲染所有参数输入框', () => {
      render(<TeacherModelParamsForm />);

      expect(screen.getByLabelText(/Temperature/i)).toBeInTheDocument();
      expect(screen.getByLabelText(/Top P/i)).toBeInTheDocument();
      expect(screen.getByLabelText(/Max Tokens/i)).toBeInTheDocument();
    });

    it('应该显示默认值', () => {
      render(<TeacherModelParamsForm />);

      const temperatureInput = screen.getByTestId('temperature-input') as HTMLInputElement;
      const topPInput = screen.getByTestId('top-p-input') as HTMLInputElement;
      const maxTokensInput = screen.getByTestId('max-tokens-input') as HTMLInputElement;

      expect(parseFloat(temperatureInput.value)).toBe(defaultTeacherSettings.temperature);
      expect(parseFloat(topPInput.value)).toBe(defaultTeacherSettings.topP);
      expect(parseInt(maxTokensInput.value)).toBe(defaultTeacherSettings.maxTokens);
    });

    it('应该显示参数范围提示', () => {
      render(<TeacherModelParamsForm />);

      expect(screen.getByText(/范围: 0 ~ 2/i)).toBeInTheDocument();
      expect(screen.getByText(/范围: 0 ~ 1/i)).toBeInTheDocument();
      expect(screen.getByText(/范围: 1 ~ 8192/i)).toBeInTheDocument();
    });
  });

  describe('输入验证', () => {
    it('应该在 temperature 超出范围时显示错误', () => {
      render(<TeacherModelParamsForm />);

      const temperatureInput = screen.getByTestId('temperature-input');
      fireEvent.change(temperatureInput, { target: { value: '3.0' } });

      // 应该显示错误信息
      expect(screen.getByText(/必须在 0 ~ 2 之间/i)).toBeInTheDocument();
    });

    it('应该在 top_p 超出范围时显示错误', () => {
      render(<TeacherModelParamsForm />);

      const topPInput = screen.getByTestId('top-p-input');
      fireEvent.change(topPInput, { target: { value: '1.5' } });

      // 应该显示错误信息
      expect(screen.getByText(/必须在 0 ~ 1 之间/i)).toBeInTheDocument();
    });

    it('应该在 max_tokens 超出范围时显示错误', () => {
      render(<TeacherModelParamsForm />);

      const maxTokensInput = screen.getByTestId('max-tokens-input');
      fireEvent.change(maxTokensInput, { target: { value: '10000' } });

      // 应该显示错误信息
      expect(screen.getByText(/必须在 1 ~ 8192 之间/i)).toBeInTheDocument();
    });

    it('应该在值恢复到有效范围后清除错误', () => {
      render(<TeacherModelParamsForm />);

      const temperatureInput = screen.getByTestId('temperature-input');
      
      // 先输入无效值
      fireEvent.change(temperatureInput, { target: { value: '3.0' } });
      expect(screen.getByText(/必须在 0 ~ 2 之间/i)).toBeInTheDocument();

      // 输入有效值
      fireEvent.change(temperatureInput, { target: { value: '1.0' } });
      expect(screen.queryByText(/必须在 0 ~ 2 之间/i)).not.toBeInTheDocument();
    });
  });

  describe('Store 集成', () => {
    it('应该在修改参数时更新 Store', () => {
      render(<TeacherModelParamsForm />);

      const temperatureInput = screen.getByTestId('temperature-input');
      fireEvent.change(temperatureInput, { target: { value: '1.5' } });

      const state = useCredentialStore.getState();
      expect(state.teacherSettings.temperature).toBe(1.5);
    });

    it('应该在修改参数时标记为脏状态', () => {
      render(<TeacherModelParamsForm />);

      // 初始状态应该是干净的
      expect(useCredentialStore.getState().isDirty).toBe(false);

      const temperatureInput = screen.getByTestId('temperature-input');
      fireEvent.change(temperatureInput, { target: { value: '1.5' } });

      // 修改后应该是脏状态
      expect(useCredentialStore.getState().isDirty).toBe(true);
    });
  });

  describe('重置按钮', () => {
    it('应该在默认值时禁用重置按钮', () => {
      render(<TeacherModelParamsForm />);

      const resetButton = screen.getByTestId('reset-teacher-settings');
      expect(resetButton).toBeDisabled();
    });

    it('应该在参数被修改后启用重置按钮', () => {
      render(<TeacherModelParamsForm />);

      const temperatureInput = screen.getByTestId('temperature-input');
      fireEvent.change(temperatureInput, { target: { value: '1.5' } });

      const resetButton = screen.getByTestId('reset-teacher-settings');
      expect(resetButton).not.toBeDisabled();
    });

    it('应该在点击重置后恢复默认值', () => {
      render(<TeacherModelParamsForm />);

      // 修改值
      const temperatureInput = screen.getByTestId('temperature-input') as HTMLInputElement;
      fireEvent.change(temperatureInput, { target: { value: '1.5' } });
      expect(parseFloat(temperatureInput.value)).toBe(1.5);

      // 点击重置
      const resetButton = screen.getByTestId('reset-teacher-settings');
      fireEvent.click(resetButton);

      // 应该恢复默认值
      expect(parseFloat(temperatureInput.value)).toBe(defaultTeacherSettings.temperature);
    });
  });
});
