/**
 * 老师模型参数配置表单
 * 用于配置 temperature、top_p、max_tokens 等参数
 */

import { useMemo } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Button } from '@/components/ui/button';
import { useCredentialStore } from '@/stores/useCredentialStore';
import { teacherSettingsConstraints, defaultTeacherSettings } from '@/types/credentials';

/**
 * 参数验证错误类型
 */
interface ValidationErrors {
  temperature?: string;
  topP?: string;
  maxTokens?: string;
}

/**
 * 根据当前参数值计算验证错误
 * Code Review Fix: 使用 useMemo 派生错误状态，而非组件内独立 useState
 * 确保与 Store 的 hasTeacherSettingsErrors 保持一致
 */
function getValidationErrors(
  temperature: number,
  topP: number,
  maxTokens: number
): ValidationErrors {
  const errors: ValidationErrors = {};
  
  const { temperature: tempConstraints, topP: topPConstraints, maxTokens: maxTokensConstraints } = teacherSettingsConstraints;
  
  if (temperature < tempConstraints.min || temperature > tempConstraints.max) {
    errors.temperature = `必须在 ${tempConstraints.min} ~ ${tempConstraints.max} 之间`;
  }
  
  if (topP < topPConstraints.min || topP > topPConstraints.max) {
    errors.topP = `必须在 ${topPConstraints.min} ~ ${topPConstraints.max} 之间`;
  }
  
  if (maxTokens < maxTokensConstraints.min || maxTokens > maxTokensConstraints.max) {
    errors.maxTokens = `必须在 ${maxTokensConstraints.min} ~ ${maxTokensConstraints.max} 之间`;
  }
  
  return errors;
}

/**
 * 老师模型参数配置表单组件
 */
export function TeacherModelParamsForm() {
  const teacherSettings = useCredentialStore((state) => state.teacherSettings);
  const setTeacherSettings = useCredentialStore((state) => state.setTeacherSettings);
  const resetTeacherSettings = useCredentialStore((state) => state.resetTeacherSettings);
  
  // Code Review Fix: 使用 useMemo 派生错误状态，确保与 Store 的 hasTeacherSettingsErrors 同步
  // 不再使用独立的 useState，避免状态不一致
  const errors = useMemo(() => 
    getValidationErrors(
      teacherSettings.temperature,
      teacherSettings.topP,
      teacherSettings.maxTokens
    ),
    [teacherSettings.temperature, teacherSettings.topP, teacherSettings.maxTokens]
  );

  const handleTemperatureChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = parseFloat(e.target.value);
    if (!isNaN(value)) {
      setTeacherSettings({ temperature: value });
    }
  };

  const handleTopPChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = parseFloat(e.target.value);
    if (!isNaN(value)) {
      setTeacherSettings({ topP: value });
    }
  };

  const handleMaxTokensChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = parseInt(e.target.value, 10);
    if (!isNaN(value)) {
      setTeacherSettings({ maxTokens: value });
    }
  };

  const isDefault = 
    teacherSettings.temperature === defaultTeacherSettings.temperature &&
    teacherSettings.topP === defaultTeacherSettings.topP &&
    teacherSettings.maxTokens === defaultTeacherSettings.maxTokens;

  return (
    <Card>
      <CardHeader>
        <CardTitle>老师模型参数</CardTitle>
        <CardDescription>
          配置老师模型的调用参数，影响模型输出的随机性和长度
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        {/* Temperature */}
        <div className="space-y-2">
          <Label htmlFor="temperature">
            Temperature
            <span className="text-muted-foreground text-sm ml-2">
              (范围: {teacherSettingsConstraints.temperature.min} ~ {teacherSettingsConstraints.temperature.max})
            </span>
          </Label>
          <Input
            id="temperature"
            type="number"
            min={teacherSettingsConstraints.temperature.min}
            max={teacherSettingsConstraints.temperature.max}
            step={teacherSettingsConstraints.temperature.step}
            value={teacherSettings.temperature}
            onChange={handleTemperatureChange}
            className={errors.temperature ? 'border-red-500' : ''}
            data-testid="temperature-input"
          />
          {errors.temperature ? (
            <p className="text-sm text-red-500">{errors.temperature}</p>
          ) : (
            <p className="text-sm text-muted-foreground">
              控制输出的随机性，值越高输出越随机
            </p>
          )}
        </div>

        {/* Top P */}
        <div className="space-y-2">
          <Label htmlFor="topP">
            Top P
            <span className="text-muted-foreground text-sm ml-2">
              (范围: {teacherSettingsConstraints.topP.min} ~ {teacherSettingsConstraints.topP.max})
            </span>
          </Label>
          <Input
            id="topP"
            type="number"
            min={teacherSettingsConstraints.topP.min}
            max={teacherSettingsConstraints.topP.max}
            step={teacherSettingsConstraints.topP.step}
            value={teacherSettings.topP}
            onChange={handleTopPChange}
            className={errors.topP ? 'border-red-500' : ''}
            data-testid="top-p-input"
          />
          {errors.topP ? (
            <p className="text-sm text-red-500">{errors.topP}</p>
          ) : (
            <p className="text-sm text-muted-foreground">
              核采样参数，控制输出的多样性
            </p>
          )}
        </div>

        {/* Max Tokens */}
        <div className="space-y-2">
          <Label htmlFor="maxTokens">
            Max Tokens
            <span className="text-muted-foreground text-sm ml-2">
              (范围: {teacherSettingsConstraints.maxTokens.min} ~ {teacherSettingsConstraints.maxTokens.max})
            </span>
          </Label>
          <Input
            id="maxTokens"
            type="number"
            min={teacherSettingsConstraints.maxTokens.min}
            max={teacherSettingsConstraints.maxTokens.max}
            step={teacherSettingsConstraints.maxTokens.step}
            value={teacherSettings.maxTokens}
            onChange={handleMaxTokensChange}
            className={errors.maxTokens ? 'border-red-500' : ''}
            data-testid="max-tokens-input"
          />
          {errors.maxTokens ? (
            <p className="text-sm text-red-500">{errors.maxTokens}</p>
          ) : (
            <p className="text-sm text-muted-foreground">
              单次输出的最大 Token 数量
            </p>
          )}
        </div>

        {/* 重置按钮 */}
        <div className="flex justify-end">
          <Button
            variant="outline"
            onClick={resetTeacherSettings}
            disabled={isDefault}
            data-testid="reset-teacher-settings"
          >
            重置为默认值
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}
